//! A very simple solution for real-time displaying system's info
//!
//! This package was built for the purpose of displaying system's information
//! in a very simple way but with relevant and real-time data

pub mod utils;
pub mod tasks;
pub mod ringbuf;
pub mod http;
pub mod schema;
pub mod systats;

extern crate sysinfo;
extern crate num_traits;
extern crate getopts;

use std::io::Error;
use std::sync::{Arc, RwLock};
use getopts::{Matches, Options};
use sysinfo::{System, SystemExt};
use crate::systats::{SysinfoStats, start_reader, init_reader};
use crate::schema::{SysinfoSchemaBuilder, SysinfoSchema};
use crate::http::server;


// const CAPACITY: usize = 120;
const CAPACITY: usize = 4;
const DEFAULT_WINDOW: u32 = 60*60; // 1 hour in seconds
const MIN_WINDOW: u32 = 8;
// const MIN_WINDOW: u32 = 10*60; // 10 minutes
const MAX_WINDOW: u32 = 24*60*60; // 24 hours

#[derive(Default, Debug)]
pub struct SysinfoOpts {
    pub sampling_freq: u32,
    pub time_window: u32,
    pub reset_flag: bool,
}
impl PartialEq for SysinfoOpts {
    fn eq(&self, other: &Self) -> bool {
        self.sampling_freq == other.sampling_freq &&
            self.time_window == other.time_window
    }
}


fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

pub fn init_opts(args: &[String]) -> Option<SysinfoOpts> {
    if args.len() == 0 {
        return None;
    }
    let program = args[0].clone();
    let mut sysopts = SysinfoOpts::default();
    let mut opts = Options::new();
    opts.optopt("t", "time", "time window period", "MINUTES");
    opts.optflag("r", "reset", "reset max and min upon new time window");
    opts.optflag("h", "help", "print this help menu");
    let matches: Matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(&program, &opts);
            return None;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return None;
    }
    if matches.opt_present("r") {
        sysopts.reset_flag = true;
    } else {
        sysopts.reset_flag = false;
    }
    if let Some(str_val) = matches.opt_str("t") {
        if let Ok(val) = str_val.parse::<u32>() {
            // let val = val*60;
            if val > MAX_WINDOW {
                sysopts.time_window = MAX_WINDOW;
            } else if val < MIN_WINDOW {
                sysopts.time_window = MIN_WINDOW;
            } else {
                sysopts.time_window = val;
            }
        } else {
            return None;
        }
    } else {
        sysopts.time_window = DEFAULT_WINDOW;
    }
    sysopts.sampling_freq = sysopts.time_window/(CAPACITY as u32);
    println!("{:?}", sysopts);

    Some(sysopts)
}

//pub fn init_sys_reader(opts: &SysinfoOpts) -> (System, SysinfoStats) {
//    let sys = System::new_all();
//    let mut sts = SysinfoStats::new(CAPACITY, opts.reset_flag);
//    let mut disks: Vec<&str> = sys.disks().iter()
//        .filter_map(|x| {
//            if let Some(x) = x.name().to_str() {
//                return Some(x);
//            }
//            None
//        })
//        .collect();
//    disks.dedup();
//    println!("DISKS: {:?}", disks);
//    let nets: Vec<&str> = sys.networks().iter()
//        .map(|(k, _v)| k.as_str())
//        .collect();
//
//    println!("NETS: {:?}", nets);
//    sts.build_dynamic_values(CAPACITY, opts.reset_flag, &disks, &nets);
//    (sys, sts)
//}

//pub fn run_sys_reader(opts: SysinfoOpts, sys: System, sts: SysinfoStats)
//                      -> Result<(), Error> {
pub fn run_sys_reader(opts: SysinfoOpts)
                      -> Result<(), Error> {
    let sys = System::new_all();
    let sts = init_reader(&opts, &sys, CAPACITY); 
    let run_flag: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
    let schema: Arc<SysinfoSchemaBuilder> = sts.get_schema_builder();
    let sys_lock: Arc<RwLock<System>> = Arc::new(RwLock::new(sys));
    let sts_lock: Arc<RwLock<SysinfoStats>> = Arc::new(RwLock::new(sts));
    //let builder = Builder::new(Arc::clone(&sts_lock));
    let h1 = start_reader(opts,
                          Arc::clone(&sys_lock),
                          Arc::clone(&sts_lock),
                          Arc::clone(&run_flag));

    //let h1 = tasks::task_sysinfo_compute(opts,
    //                                     Arc::clone(&sys_lock),
    //                                     Arc::clone(&sts_lock),
    //                                     Arc::clone(&run_flag));
    //let h2 = tasks::task_sysinfo_show(Arc::clone(&sys_lock),
    //                                  Arc::clone(&sts_lock),
    //                                  Arc::clone(&run_flag));

    let server_handler = server::start_server(Arc::clone(&schema));

    tasks::task_handle_signals(Arc::clone(&run_flag))?;
    let _ = h1.join().unwrap();
    //let _ = h2.join().unwrap();
    server::stop_server(&server_handler);
    Ok(())
}

#[test]
fn test_init_opts() {
    // Empty args
    let mut a: Vec<String> = Vec::new();
    assert!(init_opts(&a).is_none());
    a.push("sysinfo".to_string());
    a.push("-Z".to_string());
    a.push("-t".to_string());
    a.push("-h".to_string());
    // Unmapped option
    let t1 = [a[0].clone(), a[1].clone(), "10".to_string()];
    assert!(init_opts(&t1).is_none());
    // Help option should return None
    let t2 = [a[0].clone(), a[3].clone()];
    assert!(init_opts(&t2).is_none());
    // Test valid option with invalid data
    let t3 = [a[0].clone(), a[2].clone(), "str".to_string()];
    assert!(init_opts(&t3).is_none());
    // Test valid option upper bound
    let max = MAX_WINDOW + 9999;
    let freq = MAX_WINDOW/(CAPACITY as u32);
    let window = MAX_WINDOW;
    let t4 = [a[0].clone(), a[2].clone(), max.to_string()];
    assert_eq!(init_opts(&t4),
               Some(SysinfoOpts { sampling_freq: freq, time_window: window, reset_flag: true }));
    // Test valid option lower bound
    let min = MIN_WINDOW - 4;
    let freq = MIN_WINDOW/(CAPACITY as u32);
    let window = MIN_WINDOW;
    let t5 = [a[0].clone(), a[2].clone(), min.to_string()];
    assert_eq!(init_opts(&t5),
               Some(SysinfoOpts { sampling_freq: freq, time_window: window, reset_flag: true }));
    // Test allowed values
    let val: u32 = (MAX_WINDOW - MIN_WINDOW)/2;
    let freq = val/(CAPACITY as u32);
    let window = val;
    let t6 = [a[0].clone(), a[2].clone(), val.to_string()];
    assert_eq!(init_opts(&t6),
               Some(SysinfoOpts { sampling_freq: freq, time_window: window, reset_flag: true }));
}
