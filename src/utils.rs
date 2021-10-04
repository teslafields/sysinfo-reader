use std::fmt::{Formatter, Debug, Display, Result as ResultFmt};
use std::cmp::PartialOrd;
use std::ops::AddAssign;
use std::fs::File;
use std::io::prelude::*;
use std::collections::VecDeque;
use std::collections::vec_deque::Iter;
use num_traits::{Num, NumCast};
use std::iter::Sum;


pub struct RingStatsBuffer<T> {
    buff: VecDeque<T>,
    pub max: T,
    pub min: T,
    avg: T
}

impl<T> RingStatsBuffer<T> 
where T: Default + PartialOrd + Copy + Num + NumCast + AddAssign + Sum 
{
    pub fn new(capacity: usize) -> Self {
        RingStatsBuffer{
            buff: VecDeque::with_capacity(capacity),
            max: NumCast::from(u32::MIN).unwrap(),
            min: NumCast::from(u32::MAX).unwrap(),
            avg: T::default()
        }
    }

    pub fn len(&self) -> usize {
        self.buff.len()
    }

    pub fn iter(&self) -> Iter<T> {
        self.buff.iter()
    }

    pub fn push_back(&mut self, item: T) {
        if self.buff.len() >= self.buff.capacity() {
            let _ = self.buff.pop_front();
        }
        if item > self.max { self.max = item }
        if item < self.min { self.min = item }
        self.buff.push_back(item);
        self.calc_avg();
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.buff.pop_front()
    }

    pub fn get_last(&self) -> Option<T> {
        if self.buff.len() == 0 {
            return None;
        }
        let val = self.buff.get(self.buff.len()-1);
        val.copied()
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        self.buff.as_slices()
    }

    pub fn calc_avg(&mut self) {
        if self.buff.len() > 0 {
            let sum: T = self.buff.iter().copied().sum();
            self.avg = sum/NumCast::from(self.buff.len()).unwrap();
        }
    }
}

impl<T: Debug + Display> Debug for RingStatsBuffer<T> {
    fn fmt(&self, f: &mut Formatter) -> ResultFmt {
        f.write_fmt(format_args!("Min {:10.2} Max: {:10.2} Avg: {:10.2} Buff: {:?}", self.min, self.max, self.avg, self.buff))
    }
}

pub fn open_and_read(filename: &str) -> String {
    let mut f = File::open(&filename)
        .expect(&format!("Error opening file: {}", filename));
    let mut content = String::new();
    f.read_to_string(&mut content)
        .expect(&format!("Error reading content: {}", filename));
    content
}

pub fn parse_key_from_text(s: &str, k: &str, endstr: &str,
        trim: Option<&[char]>) -> Option<String> {
    match s.find(k) {
        Some(idx_b) => {
            let content = &s[idx_b + k.len()..];
            match content.find(endstr) {
                Some(idx_e) => {
                    let mut val = &content[..idx_e];
                    if trim.is_some() {
                        val = val.trim_matches(trim.unwrap());
                    }
                    Some(String::from(val))
                },
                _ => None
            }
        },
        _ => None
    }
}

pub fn parse_online_cpus(s: &str) -> Vec<u32> {
    let mut cpu_ranges: Vec<String> = Vec::new();
    let mut slice = s;
    while let Some(range) = slice.find(',') {
        cpu_ranges.push(String::from(&slice[..range]));
        slice = &slice[range + 1..];
    }
    cpu_ranges.push(String::from(slice));
    let mut cpus: Vec<u32> = Vec::new();
    for range in cpu_ranges {
        if let Some(index) = range.find('-') {
            match (&range[..index].parse::<u32>(), &range[index + 1..].parse::<u32>()) {
                (Ok(l), Ok(r)) => {
                    for i in *l..*r+1 {
                        cpus.push(i);
                    }
                },
                _ => ()
            }
        } else {
            match range.parse::<u32>() {
                Ok(nr) => { cpus.push(nr); },
                _ => ()
            }
        }
    }
    cpus
}

#[test]
fn test_parse_online_cpus() {
    assert_eq!(parse_online_cpus("0-4,6-7"), vec![0, 1, 2, 3, 4, 6, 7]);
}
