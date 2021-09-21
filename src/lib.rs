//! A very simple solution for real-time displaying system's info
//!
//! This package was built for the purpose of displaying system's information
//! in a very simple way but with relevant and real-time data

pub mod net;
pub mod mem;
pub mod cpu;
pub mod utils;
pub mod tasks;


use std::io::Error;
use std::boxed::Box;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::default::Default;


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
pub fn init_sys_reader(argv: &Vec<String>) -> SysInfoFlags {
    let mut flags = SysInfoFlags::default();
    flags.cpu = true;
    for arg in argv.iter() {
        if arg == "-m" { flags.mem = true; }
        else if arg == "-d" { flags.disk = true; }
        else if arg == "-n" { flags.net = true; }
        else if arg == "-s" { flags.sys = true; }
    }
    flags
}

/// This is a blocking function that will start the threads responsible for
/// reading and displaying the system's info in the stdout
pub fn run_sys_reader(flags: SysInfoFlags) -> Result<(), Error> {
    let mut chan_size: usize = 0;
    if flags.cpu { chan_size += 1; }
    if flags.mem { chan_size += 1; }
    if flags.disk { chan_size += 1; }
    if flags.net { chan_size += 1; }
    if flags.sys { chan_size += 1; }

    let run_flag: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
    let (tx, rx): (SyncSender<Box<dyn SysInfo + Send>>,
                   Receiver<Box<dyn SysInfo + Send>>) = sync_channel(chan_size);
    let h1 = tasks::task_read_and_send(flags, Arc::clone(&run_flag), tx);
    let h2 = tasks::task_receive_and_display(chan_size, Arc::clone(&run_flag), rx);
    tasks::task_handle_signals(Arc::clone(&run_flag))?;
    let _ = h1.join().unwrap();
    let _ = h2.join().unwrap();
    Ok(())
}
