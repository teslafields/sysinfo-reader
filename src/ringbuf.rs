use std::fmt::{Formatter, Debug, Display, Result as ResultFmt};
use std::cmp::PartialOrd;
use std::ops::AddAssign;
use std::collections::VecDeque;
use std::collections::vec_deque::Iter;
use num_traits::{Num, NumCast};
use std::iter::Sum;


pub struct RingStatsBuffer<T> {
    buff: VecDeque<T>,
    cur_max: T,
    cur_min: T,
    max: T,
    min: T,
    avg: T,
    ite: usize,
    rst: bool,
}

impl<T> RingStatsBuffer<T> 
where T: Default + PartialOrd + Copy + Num + NumCast + AddAssign + Sum 
{
    pub fn new(capacity: usize, rst: bool) -> Self {
        RingStatsBuffer{
            buff: VecDeque::with_capacity(capacity),
            avg: NumCast::from(u32::MIN).unwrap(),
            cur_max: NumCast::from(u32::MIN).unwrap(),
            cur_min: NumCast::from(u32::MAX).unwrap(),
            max: NumCast::from(0).unwrap(),
            min: NumCast::from(0).unwrap(),
            ite: 0,
            rst: rst,
        }
    }

    pub fn len(&self) -> usize {
        self.buff.len()
    }

    pub fn capacity(&self) -> usize {
        self.buff.capacity()
    }

    pub fn has_reset_flag(&self) -> bool {
        self.rst == true
    }

    pub fn iter(&self) -> Iter<T> {
        self.buff.iter()
    }

    pub fn push_back(&mut self, item: T) {
        if self.buff.len() >= self.buff.capacity() {
            let _ = self.buff.pop_front();
        }
        if item > self.cur_max { self.cur_max = item }
        if item < self.cur_min { self.cur_min = item }
        self.buff.push_back(item);
        self.ite += 1;
        if self.ite >= self.buff.capacity() {
            self.calc_stats();
            self.ite = 0;
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

    pub fn get_min(&self) -> T {
        self.cur_min
    }

    pub fn get_max(&self) -> T {
        self.cur_max
    }

    pub fn get_last_min(&self) -> T {
        self.min
    }

    pub fn get_last_max(&self) -> T {
        self.max
    }

    pub fn get_avg(&self) -> T {
        self.avg
    }

    pub fn calc_stats(&mut self) {
        if self.buff.len() > 0 {
            let sum: T = self.buff.iter().copied().sum();
            self.avg = sum/NumCast::from(self.buff.len()).unwrap();
            self.max = self.cur_max;
            self.min = self.cur_min;
            if self.rst {
                self.reset_stats();
            }
        }
    }

    fn reset_stats(&mut self) {
        self.cur_min = NumCast::from(u32::MAX).unwrap();
        self.cur_max = NumCast::from(u32::MIN).unwrap();
    }

}

impl<T: Debug + Display> Debug for RingStatsBuffer<T> {
    fn fmt(&self, f: &mut Formatter) -> ResultFmt {
        if f.alternate() {
            f.write_fmt(format_args!("Min {:10.2} Max: {:10.2} Avg: {:10.2} Buff: {:?}", self.min, self.max, self.avg, self.buff))
        } else {
            f.write_fmt(format_args!("Min {:10.2} Max: {:10.2} Avg: {:10.2}", self.min, self.max, self.avg))
        }
    }
}

#[test]
fn test_ring_stats_buffer() {
    let mut sts: RingStatsBuffer<u32> = RingStatsBuffer::new(3, false);
    sts.push_back(2);
    sts.push_back(3);
    sts.push_back(4);
    assert_eq!(sts.get_avg(), 3);
    sts.push_back(5);
    assert_eq!(sts.capacity(), 3);
    sts.calc_stats();
    assert_eq!(sts.get_avg(), 4);
    assert_eq!(sts.get_max(), 5);
    assert_eq!(sts.get_min(), 2);
    assert_eq!(sts.get_last(), Some(5));
    assert_eq!(sts.pop_front(), Some(3));
    assert_eq!(sts.len(), 2);
    let mut sts: RingStatsBuffer<u32> = RingStatsBuffer::new(3, true);
    sts.push_back(5);
    sts.push_back(1);
    sts.push_back(3);
    sts.push_back(4);
    sts.push_back(2);
    assert_eq!(sts.get_max(), 4);
    assert_eq!(sts.get_min(), 2);
    assert_eq!(sts.get_last_max(), 5);
    assert_eq!(sts.get_last_min(), 1);
}

