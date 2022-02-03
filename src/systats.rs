use std::io;
use std::io::Error;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};
use std::thread::{sleep, spawn, JoinHandle};
use sysinfo::{ProcessorExt, System, SystemExt, DiskExt, NetworkExt, NetworksExt};
use super::SysinfoOpts;
use crate::ringbuf::RingStatsBuffer;
use crate::schema::{SysinfoSchemaBuilder, SysinfoSchema};

pub struct NetworkBytes {
    pub rx_bytes: RingStatsBuffer<u64>,
    pub tx_bytes: RingStatsBuffer<u64>
}

pub struct SysinfoStats {
    schema: Arc<SysinfoSchemaBuilder>,
    pub cpu_usage: RingStatsBuffer<f32>,
    pub cpu_freq: RingStatsBuffer<u64>,
    pub mem_free: RingStatsBuffer<u64>,
    pub mem_used: RingStatsBuffer<u64>,
    pub mem_available: RingStatsBuffer<u64>,
    pub mem_buffer: RingStatsBuffer<u64>,
    pub disk_usage: HashMap<String, RingStatsBuffer<u64>>,
    pub networks: HashMap<String, NetworkBytes>,
    pub timestamp: RingStatsBuffer<u64>,
}

impl SysinfoStats {
    pub fn new(capacity: usize, rst_flag: bool) -> Self {
        SysinfoStats {
            schema: Arc::new(SysinfoSchemaBuilder::new()),
            cpu_usage: RingStatsBuffer::new(capacity, rst_flag),
            cpu_freq: RingStatsBuffer::new(capacity, rst_flag),
            mem_free: RingStatsBuffer::new(capacity, rst_flag),
            mem_used: RingStatsBuffer::new(capacity, rst_flag),
            mem_available: RingStatsBuffer::new(capacity, rst_flag),
            mem_buffer: RingStatsBuffer::new(capacity, rst_flag),
            disk_usage: HashMap::new(),
            networks: HashMap::new(),
            timestamp: RingStatsBuffer::new(capacity, rst_flag)
        }
    }

    pub fn build_dynamic_values(&mut self, capacity: usize,
                                rst_flag: bool, disks: &Vec<&str>,
                                networks: &Vec<&str>) {
        for d in disks {
            self.disk_usage.insert(d.to_string(),
                                   RingStatsBuffer::new(capacity, rst_flag));
        }
        for n in networks {
            self.networks.insert(
                n.to_string(),
                NetworkBytes {
                    rx_bytes: RingStatsBuffer::new(capacity, rst_flag),
                    tx_bytes: RingStatsBuffer::new(capacity, rst_flag)
                }
            );
        }
    }

    pub fn build_schema(&self) {
        self.schema.build(&self);
    }

    pub fn get_schema_builder(&self) -> Arc<SysinfoSchemaBuilder> {
        self.schema.clone()
    }
        
}

pub struct Builder {
    pub statslock: Arc<RwLock<SysinfoStats>>,
    // This should be reconsidered 
    pub schema: SysinfoSchema,
}

impl Builder {
    pub fn new(systats: Arc<RwLock<SysinfoStats>>) -> Self {
        Builder {
            statslock: systats,

            schema: SysinfoSchema::new()
        }
    }

    pub fn build_sysinfo_json(&self) -> Option<SysinfoSchema> {
        if let Ok(stats) = self.statslock.read() {
            let mut payload = SysinfoSchema::new();
            if stats.timestamp.len() > 0 {
                println!("TIMESTAMP:  {:?}", stats.timestamp);
                println!("CPU_USAGE:  {:?}", stats.cpu_usage);
                println!("CPU_FREQ:   {:?}", stats.cpu_freq);
                println!("MEM_FREE:   {:?}", stats.mem_free);
                println!("MEM_USED:   {:?}", stats.mem_used);
                println!("DISK_USAGE:");
                for (name, buf) in stats.disk_usage.iter() {
                    println!(" {:10}: {:?}", name, buf);
                }
                println!("NETWORKS:");
                for (name, netstat) in stats.networks.iter() {
                    println!(" {:10}: rx_bytes {:?} tx_bytes {:?}",
                             name, netstat.rx_bytes, netstat.tx_bytes);
                }
                payload.cpu.cpu_freq.max = stats.cpu_freq.get_max();
                payload.cpu.cpu_freq.min = stats.cpu_freq.get_min();
                payload.cpu.cpu_freq.avg = stats.cpu_freq.get_avg();
                payload.cpu.cpu_freq.last = stats.cpu_freq.get_last().unwrap();
                payload.cpu.cpu_usage.max = stats.cpu_usage.get_max();
                payload.cpu.cpu_usage.min = stats.cpu_usage.get_min();
                payload.cpu.cpu_usage.avg = stats.cpu_usage.get_avg();
                payload.cpu.cpu_usage.last = stats.cpu_usage.get_last().unwrap();

                return Some(payload);
            }
        }
        None
    }
}

fn read_sysinfo(sys_lock: &RwLock<System>, sts_lock: &RwLock<SysinfoStats>) {
    if let Ok(mut sys) = sys_lock.write() {
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_disks();
        let usage = sys.global_processor_info().cpu_usage();
        let freq = sys.global_processor_info().frequency();
        let fmem = sys.free_memory();
        let umem = sys.used_memory();
        let amem = sys.available_memory();
        if let Ok(mut sts) = sts_lock.write() {
            for disk in sys.disks() {
                let name = disk.name().to_str().unwrap_or("").to_string();
                if let Some(buf) = sts.disk_usage.get_mut(&name) {
                    buf.push_back(disk.total_space() -
                                  disk.available_space());
                }
            }
            for (ifname, netdata) in sys.networks().iter() {
                if let Some(netstat) = sts.networks.get_mut(ifname) {
                    netstat.tx_bytes.push_back(netdata.total_transmitted());
                    netstat.rx_bytes.push_back(netdata.total_received());
                }
            }
            sts.cpu_usage.push_back(usage);
            sts.cpu_freq.push_back(freq);
            sts.mem_free.push_back(fmem);
            sts.mem_used.push_back(umem);
            sts.mem_available.push_back(amem);
            let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => 0,
            };
            sts.timestamp.push_back(ts);
            sts.build_schema();
        }
    }
}

pub fn start_reader(sys_opts: SysinfoOpts,
                    sys_lock: Arc<RwLock<System>>,
                    sts_lock: Arc<RwLock<SysinfoStats>>,
                    run_flag: Arc<RwLock<bool>>)
                    -> JoinHandle<io::Result<()>> {
    let handle = spawn(move || {
        let read_interval = Duration::new(sys_opts.sampling_freq as u64, 0);
        let sleep_res = Duration::new(1, 0);
        let mut now = Instant::now();
        while *run_flag.read().unwrap() {
            sleep(sleep_res);
            if now.elapsed() >= read_interval {
                read_sysinfo(&sys_lock, &sts_lock);
                now = Instant::now();
            }
        }
        Ok(())
    });
    handle
}

pub fn init_reader(opts: &SysinfoOpts, sys: &System, capacity: usize) -> SysinfoStats {
    let mut sts = SysinfoStats::new(capacity, opts.reset_flag);
    let mut disks: Vec<&str> = sys.disks().iter()
        .filter_map(|x| {
            if let Some(x) = x.name().to_str() {
                return Some(x);
            }
            None
        })
        .collect();
    disks.dedup();
    let nets: Vec<&str> = sys.networks().iter()
        .map(|(k, _v)| k.as_str())
        .collect();
    sts.build_dynamic_values(capacity, opts.reset_flag, &disks, &nets);
    sts
}
