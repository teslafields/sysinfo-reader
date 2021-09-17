use std::default::Default;
use super::SysInfo;
use crate::utils;

static MEM_INFO: &str = "/proc/meminfo";

#[derive(Default)]
pub struct MemInfo {
    total: u32,
    used: u32,
    free: u32,
    buff: u32,
    swap: u32
}

impl MemInfo {
    fn read_mem_info(&mut self) {
        let content: String = utils::open_and_read(MEM_INFO);
        let key: &str = "MemTotal:";
        let total = match content.find(key) {
            Some(idx_b) => {
                let subcontent = &content[idx_b + key.len()..];
                match subcontent.find("\n") {
                    Some(idx_e) => {
                        let mut val = &subcontent[..idx_e];
                        val = val.trim_matches(
                            |c: char| c.is_ascii_whitespace() || c == ':');
                        Some(String::from(val))
                    },
                    _ => None
                }
            },
            _ => None
        };
        println!("Total mem: {}", total.unwrap_or(String::new()));
    }
}

impl SysInfo for MemInfo {
    fn new() -> Self {
        MemInfo::default()
    }

    fn read(&mut self) {
        self.read_mem_info();
    }

    fn display(&self) {
        println!("|{:=^42}|", " MEM INFO ");
        println!("| {:8} kB | {:8} kB | {:8} kB |", self.total, self.used,
                 self.free);
    }
}


