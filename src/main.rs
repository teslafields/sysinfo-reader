use std::io::Error;
use std::env;
use sysinfo_reader::*;


fn main() -> Result<(), Error> {
    let _: Vec<String> = env::args().collect();
    let sysdata = init_sys_reader(3, 2);
    run_sys_reader(sysdata)?;
    Ok(())
}

