//! A very simple solution for real-time displaying system's info
//!
//! This package was built for the purpose of displaying system's information
//! in a very simple way but with relevant and real-time data

pub mod utils;
pub mod tasks;

extern crate sysinfo;
extern crate num_traits;

use std::io::Error;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use sysinfo::{System, SystemExt, DiskExt};
use crate::utils::RingStatsBuffer;


pub struct SysinfoStats {
    pub cpu_usage: RingStatsBuffer<f32>,
    pub cpu_freq: RingStatsBuffer<u64>,
    pub mem_free: RingStatsBuffer<u64>,
    pub mem_used: RingStatsBuffer<u64>,
    pub mem_available: RingStatsBuffer<u64>,
    pub mem_buffer: RingStatsBuffer<u64>,
    pub disks_usage: HashMap<String, RingStatsBuffer<u64>>,
    pub timestamp: RingStatsBuffer<u64>,
}

impl SysinfoStats {
    pub fn new(capacity: usize) -> Self {
        SysinfoStats {
            cpu_usage: RingStatsBuffer::new(capacity),
            cpu_freq: RingStatsBuffer::new(capacity),
            mem_free: RingStatsBuffer::new(capacity),
            mem_used: RingStatsBuffer::new(capacity),
            mem_available: RingStatsBuffer::new(capacity),
            mem_buffer: RingStatsBuffer::new(capacity),
            disks_usage: HashMap::new(),
            timestamp: RingStatsBuffer::new(capacity)
        }
    }

    pub fn build_dynamic_values(&mut self, capacity: usize, disks: &Vec<String>) {
        for d in disks {
            self.disks_usage.insert(d.to_string(), RingStatsBuffer::new(capacity));
        }
    }
}

pub fn init_sys_reader(capacity: usize, interval: u64) -> (System, SysinfoStats) {
    let sys = System::new_all();
    let mut sts = SysinfoStats::new(capacity);
    let mut disks: Vec<String> = sys.disks().iter()
        .filter_map(|x| {
            if let Some(x) = x.name().to_str() {
                return Some(x.to_string());
            }
            None
        })
        .collect();
    disks.dedup();
    println!("DISKS: {:?}", disks);
    sts.build_dynamic_values(capacity, &disks);
    (sys, sts)
}

pub fn run_sys_reader(sys: System, sts: SysinfoStats) -> Result<(), Error> {
    let run_flag: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
    let sys_lock: Arc<RwLock<System>> = Arc::new(RwLock::new(sys));
    let sts_lock: Arc<RwLock<SysinfoStats>> = Arc::new(RwLock::new(sts));
    let h1 = tasks::task_sysinfo_compute(Arc::clone(&sys_lock),
                                         Arc::clone(&sts_lock),
                                         Arc::clone(&run_flag));
    let h2 = tasks::task_sysinfo_show(Arc::clone(&sys_lock),
                                      Arc::clone(&sts_lock),
                                      Arc::clone(&run_flag));
    tasks::task_handle_signals(Arc::clone(&run_flag))?;
    let _ = h1.join().unwrap();
    let _ = h2.join().unwrap();
    Ok(())
}
