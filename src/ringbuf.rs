use std::fmt::{Formatter, Debug, Display, Result as ResultFmt};
use std::cmp::PartialOrd;
use std::ops::AddAssign;
use std::collections::VecDeque;
use std::collections::vec_deque::Iter;
use num_traits::{Num, NumCast};
use std::iter::Sum;


pub struct RingStatsBuffer<T> {
    buff: VecDeque<T>,
    pub max: T,
    pub min: T,
    pub avg: T,
    acc: usize
}

impl<T> RingStatsBuffer<T> 
where T: Default + PartialOrd + Copy + Num + NumCast + AddAssign + Sum 
{
    pub fn new(capacity: usize) -> Self {
        RingStatsBuffer{
            buff: VecDeque::with_capacity(capacity),
            max: NumCast::from(u32::MIN).unwrap(),
            min: NumCast::from(u32::MAX).unwrap(),
            avg: NumCast::from(u32::MIN).unwrap(),
            acc: 0
        }
    }

    pub fn len(&self) -> usize {
        self.buff.len()
    }

    pub fn capacity(&self) -> usize {
        self.buff.capacity()
    }

    pub fn iter(&self) -> Iter<T> {
        self.buff.iter()
    }

    fn reset_stats(&mut self) {
        self.min = NumCast::from(u32::MIN).unwrap();
        self.max = NumCast::from(u32::MAX).unwrap();
        self.avg = NumCast::from(u32::MIN).unwrap();
    }

    pub fn push_back(&mut self, item: T) {
        if self.buff.len() >= self.buff.capacity() {
            let _ = self.buff.pop_front();
        }
        if item > self.max { self.max = item }
        if item < self.min { self.min = item }
        self.buff.push_back(item);
        self.acc += 1;
        if self.acc >= self.buff.capacity() {
            self.calc_avg();
            self.acc = 0;
        }
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

#[test]
fn test_ring_stats_buffer() {
    let mut sts: RingStatsBuffer<u32> = RingStatsBuffer::new(3);
    sts.push_back(2);
    sts.push_back(3);
    sts.push_back(4);
    assert_eq!(sts.avg, 3);
    sts.push_back(5);
    assert_eq!(sts.capacity(), 3);
    sts.calc_avg();
    assert_eq!(sts.avg, 4);
    assert_eq!(sts.get_last(), Some(5));
    assert_eq!(sts.pop_front(), Some(3));
    assert_eq!(sts.len(), 2);
}

