use std::fs::File;
use std::io::prelude::*;

pub fn open_and_read(filename: &str) -> String {
    let mut f = File::open(&filename)
        .expect(&format!("Error opening file: {}", filename));
    let mut content = String::new();
    f.read_to_string(&mut content)
        .expect(&format!("Error reading content: {}", filename));
    content
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
