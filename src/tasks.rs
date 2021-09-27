//! This module is responsible for to control the threads that
//! work on the main tasks, using mechanisms such as channel and
//! mutex

use std::io;
use std::io::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn, JoinHandle};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use sysinfo::{ProcessorExt, System, SystemExt};
use super::{SysinfoData, CpuStat};


pub fn task_sysinfo_compute(sysdata: Arc<RwLock<SysinfoData>>, run_flag: Arc<RwLock<bool>>)
    -> JoinHandle<io::Result<()>> {
    let handle = spawn(move || {
        let interval = match sysdata.read() {
            Ok(obj) => obj.read_interval,
            _ => 5
        };
        if let Ok(mut sysref) = sysdata.write() {
            sleep(Duration::new(1, 0));
        }
        let seconds = Duration::new(interval, 0);
        while *run_flag.read().unwrap() {
            if let Ok(mut sysref) = sysdata.write() {
                sysref.sys.refresh_cpu();
                let mut cpus: Vec<CpuStat> = Vec::new();
                for proc in sysref.sys.processors() {
                    cpus.push( CpuStat{
                        freq: proc.frequency(),
                        usage: proc.cpu_usage()
                    });
                }
                let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(n) => n.as_secs(),
                    Err(_) => 0,
                };
                // println!("{:?} {}", cpus, ts);
                sysref.cpu.push_back(cpus);
                sysref.timestamp.push_back(ts);

            }
            sleep(seconds);
        }
        Ok(())
    });
    handle
}

pub fn task_sysinfo_show(sysdata: Arc<RwLock<SysinfoData>>, run_flag: Arc<RwLock<bool>>)
        ->  JoinHandle<()> {
    let handle = spawn(move || {
        let seconds = Duration::new(2, 0);
        while *run_flag.read().unwrap() {
            if let Ok(sysref) = sysdata.read() {
                println!("{:?} {:?}", sysref.cpu, sysref.timestamp);
            }
            sleep(seconds);
        }
    });
    handle
}

/// Loop that implements signal handling
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

