mod generic;
mod cpu;
mod utils;

use std::io;
use std::io::Error;
use std::time;
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn, JoinHandle};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use crate::generic::SysInfo;


fn cpu_info_reader_thread(run_flag: Arc<RwLock<bool>>)
    // -> (Receiver<T: SysInfo>, JoinHandle<io::Result<()>>)
    -> JoinHandle<io::Result<()>>
{
    // let (tx, rx) = sync_channel(10);
    let handle = spawn(move || {
        let mut cpu_info = cpu::CpuInfo::new();
        let seconds = time::Duration::new(1, 0);
        while *run_flag.read().unwrap() {
            cpu_info.read();
            // tx.send(cpu_info);
            print!("\x1B[2J");
            cpu_info.display();
            sleep(seconds);
        }
        Ok(())
    });
    // (rx, handle)
    handle
}

fn handle_incoming_signal(run_flag: Arc<RwLock<bool>>) -> Result<(), Error> {
    let mut signals = Signals::new(&[
        SIGHUP,
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;
    for signal in signals.forever() {
        match signal as libc::c_int {
            SIGHUP | SIGTERM | SIGINT | SIGQUIT => {
                {
                    println!("Received signal {:?}", signal);
                    let mut flag = run_flag.write().unwrap();
                    *flag = false;
                    break;
                }
            },
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let run_flag: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
    let h1 = cpu_info_reader_thread(Arc::clone(&run_flag));
    handle_incoming_signal(Arc::clone(&run_flag))?;
    let _ = h1.join().unwrap();
    println!("Terminating. Bye bye");
    Ok(())
}
