//! This module implements the SysInfo trait for the Network subsystem,
//! allowing to collect and display data from network interfaces

use std::default::Default;
use std::io;
use std::fs;
use super::SysInfo;
use crate::utils;

static NET_IFACES_DIR: &str = "/sys/class/net";

#[derive(Default)]
pub struct IfaceInfo {
    name: String,
    addr: String,
    tx_bytes: u64,
    rx_bytes: u64,
}

#[derive(Default)]
pub struct NetInfo {
    ifaces: Vec<IfaceInfo>,
}

impl NetInfo {
    fn read_net_info(&mut self) -> Result<(), io::Error> {
        for entry in fs::read_dir(NET_IFACES_DIR)? {
            let entry = entry?;
            let mut iface_info = IfaceInfo::default();
            let oss_path = entry.path();
            let str_path = oss_path.to_str().unwrap();
            let oss_fname = entry.file_name();
            let str_fname = oss_fname.to_str().unwrap();
            let addr = utils::open_and_read(&format!("{}/address", str_path));
            let addr = addr.trim();
            let tx_str = utils::open_and_read(&format!("{}/statistics/tx_bytes",
                                                       str_path));
            let tx_str = tx_str.trim();
            let tx = match tx_str.parse::<u64>() {
                Ok(val) => val>>10,
                _ => 0
            };
            let rx_str = utils::open_and_read(&format!("{}/statistics/rx_bytes",
                                                       str_path));
            let rx_str = rx_str.trim();
            let rx = match rx_str.parse::<u64>() {
                Ok(val) => val>>10,
                _ => 0
            };
            iface_info.name.push_str(str_fname);
            iface_info.addr.push_str(addr);
            iface_info.tx_bytes = tx;
            iface_info.rx_bytes = rx;
            self.ifaces.push(iface_info);
        }
        Ok(())
    }
}

impl SysInfo for NetInfo {
    fn new() -> Self {
        NetInfo::default()
    }

    fn read(&mut self) {
        let _ = self.read_net_info();
    }

    fn display(&self) {
        println!("|{:=^85}|", " NET INFO ");
        let mut j = 0;
        let end = self.ifaces.len();
        for i in (0..end-1).step_by(2) {
            let iface1 = &self.ifaces[i];
            let iface2 = &self.ifaces[i+1];
            print!("|{:42}|", format!(" {} ({})", iface1.name, iface1.addr));
            println!("{:42}|", format!(" {} ({})", iface2.name, iface2.addr));
            print!("|  tx: {:11} kB | rx: {:11} kB |", iface1.tx_bytes,
                     iface1.rx_bytes);
            println!("  tx: {:11} kB | rx: {:11} kB |", iface2.tx_bytes,
                     iface2.rx_bytes);
            j += 2;
            if j < self.ifaces.len() - 1 {
                println!("|{:-^85}|", "-");
            }
        }
        if end % 2 == 1 {
            let iface = &self.ifaces[end-1];
            println!("|{:42}|", format!(" {} ({})", iface.name, iface.addr));
            println!("|  tx: {:11} kB | rx: {:11} kB |", iface.tx_bytes,
                   iface.rx_bytes);
        }

    }
}


