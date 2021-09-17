use std::io::Error;
use std::env;
use sysinfo_reader::*;


fn main() -> Result<(), Error> {
    let argv: Vec<String> = env::args().collect();
    let sys_flags = init_sys_reader(&argv);
    run_sys_reader(sys_flags)?;
    Ok(())
}

