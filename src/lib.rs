pub mod mem;
pub mod cpu;
pub mod utils;
pub mod tasks;


use std::io::Error;
use std::boxed::Box;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::default::Default;

pub trait SysInfo {
    fn new() -> Self where Self: Sized;
    fn read(&mut self);
    fn display(&self);
}

#[derive(Default)]
pub struct SysInfoFlags {
    pub cpu: bool,
    pub mem: bool,
    pub disk: bool,
    pub net: bool,
    pub sys: bool,
}

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
    let h2 = tasks::task_receive_and_display(Arc::clone(&run_flag), rx);
    tasks::task_handle_signals(Arc::clone(&run_flag))?;
    let _ = h1.join().unwrap();
    let _ = h2.join().unwrap();
    println!("Terminating. Bye bye");
    Ok(())
}
