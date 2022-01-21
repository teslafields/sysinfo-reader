//! This module is responsible for to control the threads that
//! work on the main tasks, using mechanisms such as channel and
//! mutex

use std::io;
use std::io::Error;
use std::time::{Duration, SystemTime, Instant, UNIX_EPOCH};
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn, JoinHandle};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use sysinfo::{ProcessorExt, System, SystemExt, DiskExt, NetworkExt, NetworksExt};
use super::{SysinfoOpts, SysinfoStats};


fn read_sysinfo(sys_lock: &RwLock<System>, sts_lock: &RwLock<SysinfoStats>) {
    if let Ok(mut sys) = sys_lock.write() {
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_disks();
        let usage = sys.global_processor_info().cpu_usage();
        let freq = sys.global_processor_info().frequency();
        let fmem = sys.free_memory();
        let umem = sys.used_memory();
        let amem = sys.available_memory();
        if let Ok(mut sts) = sts_lock.write() {
            for disk in sys.disks() {
                let name = disk.name().to_str().unwrap_or("").to_string();
                if let Some(buf) = sts.disk_usage.get_mut(&name) {
                    buf.push_back(disk.total_space() -
                                  disk.available_space());
                }
            }
            for (ifname, netdata) in sys.networks().iter() {
                if let Some(netstat) = sts.networks.get_mut(ifname) {
                    netstat.tx_bytes.push_back(netdata.total_transmitted());
                    netstat.rx_bytes.push_back(netdata.total_received());
                }
            }
            sts.cpu_usage.push_back(usage);
            sts.cpu_freq.push_back(freq);
            sts.mem_free.push_back(fmem);
            sts.mem_used.push_back(umem);
            sts.mem_available.push_back(amem);
            let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => 0,
            };
            sts.timestamp.push_back(ts);
        }
    }
}

pub fn task_sysinfo_compute(sys_opts: SysinfoOpts,
                            sys_lock: Arc<RwLock<System>>,
                            sts_lock: Arc<RwLock<SysinfoStats>>,
                            run_flag: Arc<RwLock<bool>>)
                            -> JoinHandle<io::Result<()>> {
    let handle = spawn(move || {
        let read_interval = Duration::new(sys_opts.sampling_freq as u64, 0);
        let sleep_res = Duration::new(1, 0);
        let mut now = Instant::now();
        while *run_flag.read().unwrap() {
            sleep(sleep_res);
            if now.elapsed() >= read_interval {
                read_sysinfo(&sys_lock, &sts_lock);
                now = Instant::now();
            }
        }
        Ok(())
    });
    handle
}

pub fn task_sysinfo_show(sys_lock: Arc<RwLock<System>>,
                         sts_lock: Arc<RwLock<SysinfoStats>>,
                         run_flag: Arc<RwLock<bool>>)
        ->  JoinHandle<()> {
    let handle = spawn(move || {
        let seconds = Duration::new(2, 0);
        while *run_flag.read().unwrap() {
            if let Ok(sys) = sys_lock.read() {
                if let Ok(sts) = sts_lock.read() {
                    if sts.timestamp.len() == 0 {
                        continue;
                    }
                    println!("========================================================");
                    println!("UPTIME: {} CPU_CORES: {} TOTAL_MEM: {} TOTAL_SWAP: {} \
                             NAME: {}", sys.uptime(),
                             sys.physical_core_count().unwrap_or(0),
                             sys.total_memory(), sys.total_swap(),
                             sys.name().unwrap_or("".to_string()));
                    println!("TIMESTAMP:  {:?}", sts.timestamp);
                    println!("CPU_USAGE:  {:?}", sts.cpu_usage);
                    println!("CPU_FREQ:   {:?}", sts.cpu_freq);
                    println!("MEM_FREE:   {:?}", sts.mem_free);
                    println!("MEM_USED:   {:?}", sts.mem_used);
                    println!("DISK_USAGE:");
                    for (name, buf) in sts.disk_usage.iter() {
                        println!(" {:10}: {:?}", name, buf);
                    }
                    println!("NETWORKS:");
                    for (name, netstat) in sts.networks.iter() {
                        println!(" {:10}: rx_bytes {:?} tx_bytes {:?}",
                                 name, netstat.rx_bytes, netstat.tx_bytes);
                    }
                }
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

