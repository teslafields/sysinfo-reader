use std::io::Error;
use std::env;
use sysinfo_reader::*;


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let opts = match init_opts(&args) {
        Some(opts) => opts,
        None => std::process::exit(1)
    };

    run_systats_reader(opts)?;
    Ok(())
}

