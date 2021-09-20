use std::default::Default;
use super::SysInfo;
use crate::utils;

static MEM_INFO: &str = "/proc/meminfo";

#[derive(Default)]
pub struct MemInfo {
    total: u32,
    used: u32,
    free: u32,
    avail: u32,
    buff: u32,
    swapt: u32,
    swapu: u32,
    swapf: u32,
}

impl MemInfo {
    fn read_mem_info(&mut self) {
        let content: String = utils::open_and_read(MEM_INFO);
        let trim: &[_] = &['\n', ' ', ':', 'k', 'B'];
        let total_str = utils::parse_key_from_text(&content,
            "MemTotal", "\n", Some(trim));
        if total_str.is_some() {
            self.total = match total_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        let free_str = utils::parse_key_from_text(&content,
            "MemFree", "\n", Some(trim));
        if free_str.is_some() {
            self.free = match free_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        let avail_str = utils::parse_key_from_text(&content,
            "MemAvailable", "\n", Some(trim));
        if avail_str.is_some() {
            self.avail = match avail_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        let cached_str = utils::parse_key_from_text(&content,
            "Cached", "\n", Some(trim));
        let buff_str = utils::parse_key_from_text(&content,
            "Buffers", "\n", Some(trim));
        let mut cached: u32 = 0;
        if cached_str.is_some() {
            cached = match cached_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        if buff_str.is_some() {
            self.buff = cached + match buff_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        let swapt_str = utils::parse_key_from_text(&content,
            "SwapTotal", "\n", Some(trim));
        if swapt_str.is_some() {
            self.swapt = match swapt_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        let swapf_str = utils::parse_key_from_text(&content,
            "SwapFree", "\n", Some(trim));
        if swapf_str.is_some() {
            self.swapf = match swapf_str.unwrap().parse::<u32>() {
                Ok(m) => m,
                _ => 0
            };
        }
        self.used = self.total - self.free - self.buff;
        self.swapu = self.swapt - self.swapf;
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
        println!("| {:11} | {:12} | {:11} |", "total (kB)", "used (kB)",
                 "free (kB)");
        println!("| {:11} | {:12} | {:11} |", self.total, self.used,
                 self.free);
        println!("|{:-^13}|{:-^14}|{:-^13}|", "-", "-", "-");
        println!("| {:11} | {:12} | {:11} |", "avail (kB)", "buff/cached", " ");
        println!("| {:11} | {:12} | {:11} |", self.avail, self.buff, " ");
        println!("|{:-^13}|{:-^14}|{:-^13}|", "-", "-", "-");
        println!("| {:11} | {:12} | {:11} |", "swap total", "swap used", "swap free");
        println!("| {:11} | {:12} | {:11} |", self.swapt, self.swapu, self.swapf);
    }
}


