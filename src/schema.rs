use std::sync::RwLock;
use std::collections::HashMap;
use num_traits::NumCast;
use serde::{ser::{Serializer, SerializeStruct}, Serialize};
use crate::systats::SysinfoStats;


pub struct Metric<T> {
    pub max: T,
    pub min: T,
    pub avg: T,
    pub last: T
}

impl<T> Metric<T> where T: NumCast {
    fn new() -> Self {
        Metric {
            max: NumCast::from(0).unwrap(),
            min: NumCast::from(0).unwrap(),
            avg: NumCast::from(0).unwrap(),
            last: NumCast::from(0).unwrap(),
        }
    }
}

impl<T> Serialize for Metric<T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Metric", 4)?;
        s.serialize_field("max", &self.max)?;
        s.serialize_field("min", &self.min)?;
        s.serialize_field("avg", &self.avg)?;
        s.serialize_field("last", &self.last)?;
        s.end()
    }
}

impl<T> Copy for Metric<T> where T: Copy {}

impl<T> Clone for Metric<T> where T: Copy {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Serialize, Clone)]
pub struct Cpu {
    pub cpu_cores: usize,
    pub cpu_freq: Metric<u64>,
    pub cpu_usage: Metric<f32>,
}

#[derive(Serialize, Clone)]
pub struct Mem {
    pub total_mem: u64,
    pub total_swap: u64,
    pub mem_free: Metric<u64>,
    pub mem_used: Metric<u64>,
    pub mem_available: Metric<u64>,
    pub mem_buffer: Metric<u64>,
}

#[derive(Serialize, Clone)]
pub struct Info {
    pub uptime: u64,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct Net {
    rx_bytes: Metric<u64>,
    tx_bytes: Metric<u64>
}

impl Net {
    pub fn new() -> Self {
        Net {
            rx_bytes: Metric::new(),
            tx_bytes: Metric::new(),
        }
    }
}

type Disk = HashMap<String, Metric<u64>>;
type Networks = HashMap<String, Net>;

#[derive(Serialize, Clone)]
pub struct SysinfoSchema {
    pub cpu: Cpu,
    pub mem: Mem,
    pub system: Info,
    pub disks: Disk,
    pub networks: Networks,
}

impl SysinfoSchema {
    pub fn new() -> Self {
        SysinfoSchema {
            cpu: Cpu {
                cpu_cores: 0,
                cpu_freq: Metric::new(),
                cpu_usage: Metric::new(),
            },
            mem: Mem {
                total_mem: 0,
                total_swap: 0,
                mem_free: Metric::new(),
                mem_used: Metric::new(),
                mem_available: Metric::new(),
                mem_buffer: Metric::new(),
            },
            system: Info {
                uptime: 0,
                name: String::new(),
            },
            disks: HashMap::new(),
            networks: HashMap::new(),
        }
    }
}

pub struct SysinfoSchemaBuilder {
    schema_lock: RwLock<SysinfoSchema>,
    //schema: SysinfoSchema,
}

impl SysinfoSchemaBuilder {
    pub fn new() -> Self {
        SysinfoSchemaBuilder {
            schema_lock: RwLock::new(SysinfoSchema::new()),
        }
    }

    pub fn build(&self, stats: &SysinfoStats) {
        if let Ok(mut schema) = self.schema_lock.write() {
            schema.system.uptime = stats.uptime;
            if schema.system.name.is_empty() {
                schema.system.name = stats.name.clone();
            }
            schema.cpu.cpu_cores = stats.cpu_cores;
            schema.mem.total_mem = stats.total_mem;
            schema.mem.total_swap = stats.total_swap;
            schema.cpu.cpu_freq.max = stats.cpu_freq.get_max();
            schema.cpu.cpu_freq.min = stats.cpu_freq.get_min();
            schema.cpu.cpu_freq.avg = stats.cpu_freq.get_avg();
            schema.cpu.cpu_freq.last = stats.cpu_freq.get_last().unwrap();
            schema.cpu.cpu_usage.max = stats.cpu_usage.get_max();
            schema.cpu.cpu_usage.min = stats.cpu_usage.get_min();
            schema.cpu.cpu_usage.avg = stats.cpu_usage.get_avg();
            schema.cpu.cpu_usage.last = stats.cpu_usage.get_last().unwrap();
            schema.mem.mem_free.max = stats.mem_free.get_max();
            schema.mem.mem_free.min = stats.mem_free.get_min();
            schema.mem.mem_free.avg = stats.mem_free.get_avg();
            schema.mem.mem_free.last = stats.mem_free.get_last().unwrap();
            schema.mem.mem_used.max = stats.mem_used.get_max();
            schema.mem.mem_used.min = stats.mem_used.get_min();
            schema.mem.mem_used.avg = stats.mem_used.get_avg();
            schema.mem.mem_used.last = stats.mem_used.get_last().unwrap();
            schema.mem.mem_available.max = stats.mem_available.get_max();
            schema.mem.mem_available.min = stats.mem_available.get_min();
            schema.mem.mem_available.avg = stats.mem_available.get_avg();
            schema.mem.mem_available.last = stats.mem_available.get_last().unwrap();
            for (label, diskdata) in stats.disk_usage.iter() {
                if !schema.disks.contains_key(label) {
                    schema.disks.insert(label.clone(), Metric::new());
                }
                if let Some(diskschema) = schema.disks.get_mut(label) {
                    diskschema.max = diskdata.get_max();
                    diskschema.min = diskdata.get_min();
                    diskschema.avg = diskdata.get_avg();
                    diskschema.last = diskdata.get_last().unwrap();
                }
            }
            for (label, netdata) in stats.networks.iter() {
                if !schema.networks.contains_key(label) {
                    schema.networks.insert(label.clone(), Net::new());
                }
                if let Some(netschema) = schema.networks.get_mut(label) {
                    netschema.rx_bytes.max = netdata.rx_bytes.get_max();
                    netschema.rx_bytes.min = netdata.rx_bytes.get_min();
                    netschema.rx_bytes.avg = netdata.rx_bytes.get_avg();
                    netschema.rx_bytes.last = netdata.rx_bytes.get_last().unwrap();
                    netschema.tx_bytes.max = netdata.tx_bytes.get_max();
                    netschema.tx_bytes.min = netdata.tx_bytes.get_min();
                    netschema.tx_bytes.avg = netdata.tx_bytes.get_avg();
                    netschema.tx_bytes.last = netdata.tx_bytes.get_last().unwrap();
                }
            }
        }
    }

    pub fn get_full_payload(&self) -> Option<SysinfoSchema> {
        if let Ok(schema) = self.schema_lock.read() {
            let payload = schema.clone();
            return Some(payload);
        }
        None
    }

    pub fn get_cpu_payload(&self) -> Option<Cpu> {
        if let Ok(schema) = self.schema_lock.read() {
            let payload = schema.cpu.clone();
            return Some(payload);
        }
        None
    }

    pub fn get_mem_payload(&self) -> Option<Mem> {
        if let Ok(schema) = self.schema_lock.read() {
            let payload = schema.mem.clone();
            return Some(payload);
        }
        None
    }

    pub fn get_disks_payload(&self) -> Option<Disk> {
        if let Ok(schema) = self.schema_lock.read() {
            let payload = schema.disks.clone();
            return Some(payload);
        }
        None
    }

    pub fn get_networks_payload(&self) -> Option<Networks> {
        if let Ok(schema) = self.schema_lock.read() {
            let payload = schema.networks.clone();
            return Some(payload);
        }
        None
    }

}
