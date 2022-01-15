use std::io::Error;
use std::env;
use sysinfo_reader::*;


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let opts = match init_opts(&args) {
        Some(opts) => opts,
        None => std::process::exit(1)
    };
    //std::process::exit(1);
    let (sys, sts) = init_sys_reader(&opts);
    run_sys_reader(opts, sys, sts)?;
    Ok(())
}

