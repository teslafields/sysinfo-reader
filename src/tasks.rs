use std::io;
use std::io::Error;
use std::time;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::{sleep, spawn, JoinHandle};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use crate::cpu::CpuInfo;
use crate::mem::MemInfo;
use crate::net::NetInfo;
use super::{SysInfo, SysInfoFlags};


pub fn task_read_and_send(sys_flags: SysInfoFlags, run_flag: Arc<RwLock<bool>>,
    tx: SyncSender<Box<dyn SysInfo + Send>>) -> JoinHandle<io::Result<()>>
{
    let handle = spawn(move || {
        let seconds = time::Duration::new(1, 0);
        while *run_flag.read().unwrap() {
            let mut data_out: Vec<Box<dyn SysInfo + Send>> = Vec::new();
            if sys_flags.cpu {
                let mut cpu_info = CpuInfo::new();
                cpu_info.read();
                data_out.push(Box::new(cpu_info));
            }
            if sys_flags.mem {
                let mut mem_info = MemInfo::new();
                mem_info.read();
                data_out.push(Box::new(mem_info));
            }
            if sys_flags.net {
                let mut net_info = NetInfo::new();
                net_info.read();
                data_out.push(Box::new(net_info));
            }
            if data_out.len() > 0 {
                for data in data_out {
                    if let Some(err) = tx.send(data).err() {
                        println!("Error in tx: {:?}", err);
                    }
                }
            }
            sleep(seconds);
        }
        Ok(())
    });
    handle
}

pub fn task_receive_and_display(chan_size: usize, run_flag: Arc<RwLock<bool>>,
        rx: Receiver<Box<dyn SysInfo + Send>>)
        ->  JoinHandle<()> {
    let handle = spawn(move || {
        while *run_flag.read().unwrap() {
            let mut data_in: Vec<Box<dyn SysInfo + Send>> = Vec::new();
            for _i in 0..chan_size {
                match rx.recv() {
                    Ok(d) => data_in.push(d),
                    Err(_) => (),
                }
            }
            print!("\x1B[2J");
            for d in data_in {
                d.display();
            }
            println!("|{:=^85}|", "=");

        }
    });
    handle
}

pub fn task_handle_signals(run_flag: Arc<RwLock<bool>>) -> Result<(), Error> {
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

