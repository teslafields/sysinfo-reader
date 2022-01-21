use num_traits::{Num, NumCast};
use serde::{ser::{Serializer, SerializeStruct}, Serialize};


pub struct Metric<T> {
    pub max: T,
    pub min: T,
    pub avg: T,
    pub last: T
}

impl<T> Metric<T> where T: Default + Serialize + NumCast {
    fn new() -> Self {
        Metric {
            max: NumCast::from(0).unwrap(),
            min: NumCast::from(0).unwrap(),
            avg: NumCast::from(0).unwrap(),
            last: NumCast::from(0).unwrap(),
        }
    }
}

impl<T> Serialize for Metric<T> where T: Default + Serialize + NumCast {
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


#[derive(Serialize)]
pub struct Cpu {
    pub cpu_freq: Metric<u64>,
    pub cpu_usage: Metric<f32>,
}

#[derive(Serialize)]
pub struct Mem {
    pub mem_free: Metric<u64>,
    pub mem_used: Metric<u64>,
    pub mem_available: Metric<u64>,
    pub mem_buffer: Metric<u64>,
}

#[derive(Serialize)]
pub struct SysinfoPayload {
    pub cpu: Cpu,
    pub mem: Mem,
}

//impl<T, U> SysinfoPayload<T, U> where 
//    T: Default + Serialize + NumCast,
//    U: Default + Serialize + NumCast {
impl SysinfoPayload {
    pub fn new() -> Self {
        SysinfoPayload {
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
