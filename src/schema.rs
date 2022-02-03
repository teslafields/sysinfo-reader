use std::sync::RwLock;
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
    pub cpu_freq: Metric<u64>,
    pub cpu_usage: Metric<f32>,
}

#[derive(Serialize, Clone)]
pub struct Mem {
    pub mem_free: Metric<u64>,
    pub mem_used: Metric<u64>,
    pub mem_available: Metric<u64>,
    pub mem_buffer: Metric<u64>,
}

#[derive(Serialize, Clone)]
pub struct SysinfoSchema {
    pub cpu: Cpu,
    pub mem: Mem,
}

impl SysinfoSchema {
    pub fn new() -> Self {
        SysinfoSchema {
            cpu: Cpu {
               cpu_freq: Metric::new(),
               cpu_usage: Metric::new(),
            },
            mem: Mem {
               mem_free: Metric::new(),
               mem_used: Metric::new(),
               mem_available: Metric::new(),
               mem_buffer: Metric::new(),
            },
        }
    }
}

pub struct SysinfoSchemaBuilder {
    schema_lock: RwLock<SysinfoSchema>,
    //schema: SysinfoSchema,
}

//impl<T, U> SysinfoPayload<T, U> where 
//    T: Default + Serialize + NumCast,
//    U: Default + Serialize + NumCast {
impl SysinfoSchemaBuilder {
    pub fn new() -> Self {
        SysinfoSchemaBuilder {
            schema_lock: RwLock::new(SysinfoSchema::new()),
        }
    }

    pub fn build(&self, stats: &SysinfoStats) {
        if let Ok(mut schema) = self.schema_lock.write() {
            schema.cpu.cpu_freq.max = stats.cpu_freq.get_max();
            schema.cpu.cpu_freq.min = stats.cpu_freq.get_min();
            schema.cpu.cpu_freq.avg = stats.cpu_freq.get_avg();
            schema.cpu.cpu_freq.last = stats.cpu_freq.get_last().unwrap();
            schema.cpu.cpu_usage.max = stats.cpu_usage.get_max();
            schema.cpu.cpu_usage.min = stats.cpu_usage.get_min();
            schema.cpu.cpu_usage.avg = stats.cpu_usage.get_avg();
            schema.cpu.cpu_usage.last = stats.cpu_usage.get_last().unwrap();
        }
    }

    pub fn get_payload(&self) -> Option<SysinfoSchema> {
        if let Ok(schema) = self.schema_lock.read() {
            let payload = schema.clone();
            return Some(payload);
        }
        None
    }
}
