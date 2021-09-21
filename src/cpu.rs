//! This module implements the SysInfo trait for the CPU subsystem,
//! allowing to collect and display data from CPU

use std::default::Default;
use super::SysInfo;
use crate::utils;

static CPU_INFO: &str = "/proc/cpuinfo";
static CPU_ONLINE: &str = "/sys/devices/system/cpu/online";
static CPU_FREQ: &str = "/sys/devices/system/cpu/cpu@/cpufreq/scaling_cur_freq";
static CPU_DRIVER: &str = "/sys/devices/system/cpu/cpu@/cpufreq/scaling_driver";
static CPU_GOVERNOR: &str = "/sys/devices/system/cpu/cpu@/cpufreq/scaling_governor";

#[derive(Default)]
struct Cpu {
    id: u32,
    governor: String,
    driver: String,
    freq: f64
}

#[derive(Default)]
pub struct CpuInfo {
    model: String,
    cpus: Vec<Cpu>
}

impl CpuInfo {
    fn read_cpus_online(&mut self) {
        let content: String = utils::open_and_read(CPU_ONLINE);
        let content = content.trim();
        let cpus_online = utils::parse_online_cpus(content);
        for cpu in cpus_online {
            let mut cpui = Cpu::default();
            cpui.id = cpu;
            self.cpus.push(cpui);
        }
    }

    fn read_model() -> String {
        let content: String = utils::open_and_read(CPU_INFO);
        let key: &str = "model name";
        let cpu_model = match content.find(key) {
            Some(idx_b) => {
                let subcontent = &content[idx_b + key.len()..];
                match subcontent.find("\n") {
                    Some(idx_e) => {
                        let mut model = &subcontent[..idx_e];
                        model = model.trim_matches(
                            |c: char| c.is_ascii_whitespace() || c == ':');
                        Some(String::from(model))
                    },
                    _ => None
                }
            },
            _ => None
        };
        cpu_model.unwrap_or(String::new())
    }

    fn read_freq(id: &u32) -> f64 {
        let filename = CPU_FREQ.replace("@", &*id.to_string());
        let content: String = utils::open_and_read(&filename);
        match content.trim().parse::<f64>() {
            Ok(val) => val/1000.,
            _ => 0.
        }
    }

    fn read_driver(id: &u32) -> String {
        let filename = CPU_DRIVER.replace("@", &*id.to_string());
        let content: String = utils::open_and_read(&filename);
        String::from(content.trim())
    }

    fn read_governor(id: &u32) -> String {
        let filename = CPU_GOVERNOR.replace("@", &*id.to_string());
        let content: String = utils::open_and_read(&filename);
        String::from(content.trim())
    }
}

impl SysInfo for CpuInfo {
    fn new() -> Self {
        CpuInfo::default()
    }

    fn read(&mut self) {
        self.model = CpuInfo::read_model();
        self.read_cpus_online();
        for cpu in self.cpus.iter_mut() {
            cpu.freq = CpuInfo::read_freq(&cpu.id);
            cpu.driver = CpuInfo::read_driver(&cpu.id);
            cpu.governor = CpuInfo::read_governor(&cpu.id);
        }
    }

    fn display(&self) {
        println!("|{:=^85}|", " CPU INFO ");
        println!("| {:83} |", self.model);
        let end = self.cpus.len();
        for i in (0..end-1).step_by(2) {
            let cpu1 = &self.cpus[i];
            let cpu2 = &self.cpus[i+1];
            println!("|{:-^42}|{:-^42}|", format!(" CPU{} ", cpu1.id), format!(" CPU{} ", cpu2.id));
            println!("| {:7.2} MHz | {:11} | {:11} | {:7.2} MHz | {:11} | {:11} |",
                     cpu1.freq, cpu1.governor, cpu1.driver, cpu2.freq, cpu2.governor,
                     cpu2.driver);
        }
        if end % 2 == 1 {
            let cpu = &self.cpus[end-1];
            println!("|{:-^42}|", format!(" CPU{} ", cpu.id));
            println!("| {:7.2} MHz | {:11} | {:11} |",
                     cpu.freq, cpu.governor, cpu.driver);
        }
    }
}


