//! A very simple solution for real-time displaying system's info
//!
//! This package was built for the purpose of displaying system's information
//! in a very simple way but with relevant and real-time data

pub mod net;
pub mod mem;
pub mod cpu;
pub mod utils;
pub mod tasks;

extern crate sysinfo;

use std::io::Error;
use std::sync::{Arc, RwLock};
use std::fmt;
use std::default::Default;
use sysinfo::{System, SystemExt};
use crate::utils::RingBuffer;


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
    cpu: RingBuffer<Vec<CpuStat>>,
    timestamp: RingBuffer<u64>,
    read_interval: u64,
}

/// The generic trait for all subsystems
pub trait SysInfo {
    fn new() -> Self where Self: Sized;
    fn read(&mut self);
    fn display(&self);
}

/// Flags that control which subsystem will be active
#[derive(Default)]
pub struct SysInfoFlags {
    pub cpu: bool,
    pub mem: bool,
    pub disk: bool,
    pub net: bool,
    pub sys: bool,
}

/// This function initialize the program by returning a SysInfoFlags struct based
/// on the provided command-line arguments
pub fn init_sys_reader(capacity: usize, interval: u64) -> SysinfoData {
    SysinfoData {
        sys: System::new_all(),
        cpu: RingBuffer::new(capacity),
        timestamp: RingBuffer::new(capacity),
        read_interval: interval
    }
}

/// This is a blocking function that will start the threads responsible for
/// reading and displaying the system's info in the stdout
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
