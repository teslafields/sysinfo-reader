use std::io;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};
use std::thread::{sleep, spawn, JoinHandle};
use sysinfo::{ProcessorExt, System, SystemExt, DiskExt, NetworkExt, NetworksExt};
use crate::ringbuf::RingStatsBuffer;
use crate::schema::SysinfoSchemaBuilder;


pub struct NetworkBytes {
    pub rx_bytes: RingStatsBuffer<u64>,
    pub tx_bytes: RingStatsBuffer<u64>
}

pub struct SysinfoStats {
    pub name: String,
    pub uptime: u64,
    pub cpu_cores: usize,
    pub total_mem: u64,
    pub total_swap: u64,
    pub cpu_usage: RingStatsBuffer<f32>,
    pub cpu_freq: RingStatsBuffer<u64>,
    pub mem_free: RingStatsBuffer<u64>,
    pub mem_used: RingStatsBuffer<u64>,
    pub mem_available: RingStatsBuffer<u64>,
    pub disk_usage: HashMap<String, RingStatsBuffer<u64>>,
    pub networks: HashMap<String, NetworkBytes>,
    pub timestamp: RingStatsBuffer<u64>,
}

impl SysinfoStats {
    pub fn new(capacity: usize, rst_flag: bool) -> Self {
        SysinfoStats {
            name: String::new(),
            uptime: 0,
            cpu_cores: 0,
            total_mem: 0,
            total_swap: 0,
            cpu_usage: RingStatsBuffer::new(capacity, rst_flag),
            cpu_freq: RingStatsBuffer::new(capacity, rst_flag),
            mem_free: RingStatsBuffer::new(capacity, rst_flag),
            mem_used: RingStatsBuffer::new(capacity, rst_flag),
            mem_available: RingStatsBuffer::new(capacity, rst_flag),
            disk_usage: HashMap::new(),
            networks: HashMap::new(),
            timestamp: RingStatsBuffer::new(capacity, rst_flag)
        }
    }

    pub fn build_dynamic_values(&mut self, disks: &Vec<&str>,
                                networks: &Vec<&str>) {
        let capacity = self.timestamp.capacity();
        let rst_flag = self.timestamp.has_reset_flag();
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
    sampling_freq: u64,
    // In the future, check for a Fn pointer
    schema: Arc<SysinfoSchemaBuilder>,
}

impl SystatsExecutor {
    pub fn new(capacity: usize, sampling_freq: u64, reset_flag: bool,
               schema: Arc<SysinfoSchemaBuilder>) -> Self {
        SystatsExecutor {
            systats: SysinfoStats::new(capacity, reset_flag),
            sampling_freq: sampling_freq,
            schema: schema,
        }
    }

    #[cfg(feature = "debug_systats")]
    fn debug_systats(&self) {
        println!("========================================================");
        println!("UPTIME: {} CPU_CORES: {} TOTAL_MEM: {} TOTAL_SWAP: {} \
                 NAME: {}", self.systats.uptime, self.systats.cpu_cores,
                 self.systats.total_mem, self.systats.total_swap,
                 self.systats.name);
        println!("TIMESTAMP:  {:?}", self.systats.timestamp);
        println!("CPU_USAGE:  {:?}", self.systats.cpu_usage);
        println!("CPU_FREQ:   {:?}", self.systats.cpu_freq);
        println!("MEM_FREE:   {:?}", self.systats.mem_free);
        println!("MEM_USED:   {:?}", self.systats.mem_used);
        println!("DISK_USAGE:");
        for (name, buf) in self.systats.disk_usage.iter() {
            println!(" {:10}: {:?}", name, buf);
        }
        println!("NETWORKS:");
        for (name, netstat) in self.systats.networks.iter() {
            println!(" {:8}:\n  Rx {:?}\n  Tx {:?}",
                     name, netstat.rx_bytes, netstat.tx_bytes);
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
        if self.systats.name.is_empty() {
            self.systats.name = sysinfo.name().unwrap_or("".to_string());
        }
        self.systats.uptime = sysinfo.uptime();
        self.systats.cpu_cores = sysinfo.physical_core_count().unwrap_or(0);
        self.systats.total_mem = sysinfo.total_memory();
        self.systats.total_swap = sysinfo.total_swap();
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
        #[cfg(feature = "debug_systats")]
        {
            self.debug_systats();
        }
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
        self.systats.build_dynamic_values(&disks, &nets);
    }

    pub fn run_executor(mut self, mut sysinfo: System,
                        run_flag: Arc<RwLock<bool>>)
                        -> JoinHandle<io::Result<()>> {
        self.init_dynamic_attrs(&sysinfo);
        let handle = spawn(move || {
            let read_interval = Duration::new(self.sampling_freq, 0);
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

