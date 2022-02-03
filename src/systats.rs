use std::io;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};
use std::thread::{sleep, spawn, JoinHandle};
use sysinfo::{ProcessorExt, System, SystemExt, DiskExt, NetworkExt, NetworksExt};
use super::SysinfoOpts;
use crate::ringbuf::RingStatsBuffer;
use crate::schema::SysinfoSchemaBuilder;


const CAPACITY: usize = 4;

pub struct NetworkBytes {
    pub rx_bytes: RingStatsBuffer<u64>,
    pub tx_bytes: RingStatsBuffer<u64>
}

pub struct SysinfoStats {
    // schema: Arc<SysinfoSchemaBuilder>,
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
            // schema: Arc::new(SysinfoSchemaBuilder::new()),
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
}


pub struct SystatsExecutor {
    systats: SysinfoStats,
    sysopts: SysinfoOpts,
    // In the future, check for a Fn pointer
    schema: Arc<SysinfoSchemaBuilder>,
}

impl SystatsExecutor {
    pub fn new(opts: SysinfoOpts, schema: Arc<SysinfoSchemaBuilder>) -> Self {
        SystatsExecutor {
            systats: SysinfoStats::new(CAPACITY, opts.reset_flag),
            sysopts: opts, 
            schema: schema,
        }
    }

    fn read_sysinfo(&mut self, sysinfo: &mut System) {
        sysinfo.refresh_cpu();
        sysinfo.refresh_memory();
        sysinfo.refresh_disks();
        let usage = sysinfo.global_processor_info().cpu_usage();
        let freq = sysinfo.global_processor_info().frequency();
        let fmem = sysinfo.free_memory();
        let umem = sysinfo.used_memory();
        let amem = sysinfo.available_memory();
        for disk in sysinfo.disks() {
            let name = disk.name().to_str().unwrap_or("").to_string();
            if let Some(buf) = self.systats.disk_usage.get_mut(&name) {
                buf.push_back(disk.total_space() -
                              disk.available_space());
            }
        }
        for (ifname, netdata) in sysinfo.networks().iter() {
            if let Some(netstat) = self.systats.networks.get_mut(ifname) {
                netstat.tx_bytes.push_back(netdata.total_transmitted());
                netstat.rx_bytes.push_back(netdata.total_received());
            }
        }
        self.systats.cpu_usage.push_back(usage);
        self.systats.cpu_freq.push_back(freq);
        self.systats.mem_free.push_back(fmem);
        self.systats.mem_used.push_back(umem);
        self.systats.mem_available.push_back(amem);
        let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => 0,
        };
        self.systats.timestamp.push_back(ts);
        self.schema.build(&self.systats);
    }

    fn init_dynamic_attrs(&mut self, sysinfo: &System) {
        let mut disks: Vec<&str> = sysinfo.disks().iter()
            .filter_map(|x| {
                if let Some(x) = x.name().to_str() {
                    return Some(x);
                }
                None
            })
            .collect();
        disks.dedup();
        let nets: Vec<&str> = sysinfo.networks().iter()
            .map(|(k, _v)| k.as_str())
            .collect();
        self.systats.build_dynamic_values(CAPACITY, self.sysopts.reset_flag,
                                          &disks, &nets);
    }

    pub fn run_executor(mut self, mut sysinfo: System,
                        run_flag: Arc<RwLock<bool>>)
                        -> JoinHandle<io::Result<()>> {
        self.init_dynamic_attrs(&sysinfo);
        let sampling_freq = self.sysopts.sampling_freq as u64;
        let handle = spawn(move || {
            let read_interval = Duration::new(sampling_freq, 0);
            let sleep_res = Duration::new(1, 0);
            let mut now = Instant::now();
            while *run_flag.read().unwrap() {
                sleep(sleep_res);
                if now.elapsed() >= read_interval {
                    self.read_sysinfo(&mut sysinfo);
                    now = Instant::now();
                }
            }
            Ok(())
        });
        handle
    }
}

