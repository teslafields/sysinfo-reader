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
use std::fmt;
use std::default::Default;
use std::cmp::PartialOrd;
use std::iter::Sum;
use num::{Num, NumCast};
use sysinfo::{System, SystemExt};
use crate::utils::RingBuffer;
use std::ops::{Add, Mul, Div, Sub, AddAssign};


struct Stat<T> {
    buff: RingBuffer<T>,
    pub max: T,
    pub min: T,
    avg: T
}

impl<T> Stat<T> 
where T: Default + PartialOrd + Copy + Num + NumCast + AddAssign + Sum
{
    pub fn new(capacity: usize) -> Self {
        Stat {
            buff: RingBuffer::new(capacity),
            max: NumCast::from(u32::MIN).unwrap(),
            min: NumCast::from(u32::MAX).unwrap(),
            avg: T::default()
        }   
    }   

    pub fn push_value(&mut self, val: T) {
        if val > self.max { self.max = val }
        if val < self.min { self.min = val }
        self.buff.push_back(val);
        let sum: T = self.buff.iter().copied().sum();
        self.avg = sum/NumCast::from(self.buff.length()).unwrap();
    }
}

struct SysinfoStats {
    cpu_usage: Stat<f32>,
    cpu_freq: Stat<u64>,
    mem_free: Stat<u64>,
    mem_used: Stat<u64>,
    timestamp: RingBuffer<u64>,
}

impl SysinfoStats {
    pub fn new(capacity: usize) -> Self {
        SysinfoStats {
            cpu_usage: Stat::new(capacity),
            cpu_freq: Stat::new(capacity),
            mem_free: Stat::new(capacity),
            mem_available: Stat::new(capacity),
            mem_buffer: Stat::new(capacity),
            timestamp: RingBuffer::new(capacity)
        }
    }
}

struct CpuStat {
    freq: u64,
    usage: f32
}

impl fmt::Debug for CpuStat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("").field(&self.freq).field(&self.usage).finish()
    }
}

pub struct SysinfoData {
    sys: System,
    stats: SysinfoStats,
    read_interval: u64,
}

pub fn init_sys_reader(capacity: usize, interval: u64) -> SysinfoData {
    SysinfoData {
        sys: System::new_all(),
        stats: SysinfoStats::new(capacity),
        read_interval: interval
    }
}

pub fn run_sys_reader(sysdata: SysinfoData) -> Result<(), Error> {
    let run_flag: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
    let sys_arc: Arc<RwLock<SysinfoData>> = Arc::new(RwLock::new(sysdata));
    let h1 = tasks::task_sysinfo_compute(Arc::clone(&sys_arc), Arc::clone(&run_flag));
    let h2 = tasks::task_sysinfo_show(Arc::clone(&sys_arc), Arc::clone(&run_flag));
    tasks::task_handle_signals(Arc::clone(&run_flag))?;
    let _ = h1.join().unwrap();
    let _ = h2.join().unwrap();
    Ok(())
}
