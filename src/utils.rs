use std::fmt;
use std::iter::IntoIterator;
use std::fs::File;
use std::io::prelude::*;
use std::collections::VecDeque;
use std::collections::vec_deque::Iter;


pub struct RingBuffer<T>(VecDeque<T>, usize);

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer(VecDeque::with_capacity(capacity), capacity)
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<T> {
        self.0.iter()
    }

    pub fn push_back(&mut self, item: T) {
        if self.0.len() >= self.1 {
            let _ = self.0.pop_front();
        }
        self.0.push_back(item);
    }

    pub fn pop_pront(&mut self) {
        let _ = self.0.pop_front();
    }
}

impl<'a, T> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.0.iter()
    }
}

impl<T: fmt::Debug> fmt::Debug for RingBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
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
