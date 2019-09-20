use std::ops::{Range, RangeFrom, RangeInclusive};

pub trait IntoBuffer: Sized {
    type Into: Buffer<Self>;
}

pub trait Buffer<Source> {
    type Item;

    fn from_source(source: Source) -> Self;

    fn get(&mut self, index: usize) -> Option<Self::Item>;
}

impl<'a, T> IntoBuffer for std::slice::Iter<'a, T> {
    type Into = &'a [T];
}

impl<'a, T> Buffer<std::slice::Iter<'a, T>> for &'a [T] {
    type Item = &'a T;

    fn from_source(source: std::slice::Iter<'a, T>) -> Self {
        source.as_slice()
    }

    fn get(&mut self, index: usize) -> Option<Self::Item> {
        <[T]>::get(self, index)
    }
}

impl<T: Clone> IntoBuffer for std::vec::IntoIter<T> {
    type Into = Vec<T>;
}

impl<T: Clone> Buffer<std::vec::IntoIter<T>> for Vec<T> {
    type Item = T;

    fn from_source(source: std::vec::IntoIter<T>) -> Self {
        source.collect()
    }

    fn get(&mut self, index: usize) -> Option<Self::Item> {
        if index < self.len() {
            Some(self[index].clone())
        } else {
            None
        }
    }
}

macro_rules! impl_range {
    ($range:ident, $get_start:ident, $num:ty) => {
        impl IntoBuffer for $range<$num> {
            type Into = Self;
        }

        impl Buffer<$range<$num>> for $range<$num> {
            type Item = $num;

            fn from_source(source: $range<$num>) -> Self {
                source
            }

            fn get(&mut self, index: usize) -> Option<Self::Item> {
                let val = $get_start(self) + index as $num;

                if self.contains(&val) {
                    Some(val)
                } else {
                    None
                }
            }
        }
    };
}

macro_rules! impl_range_all_nums {
    ($range:ident, $get_start:tt) => {
        impl_range!($range, $get_start, u8);
        impl_range!($range, $get_start, u16);
        impl_range!($range, $get_start, u32);
        impl_range!($range, $get_start, u64);
        impl_range!($range, $get_start, u128);
        impl_range!($range, $get_start, usize);
        impl_range!($range, $get_start, i8);
        impl_range!($range, $get_start, i16);
        impl_range!($range, $get_start, i32);
        impl_range!($range, $get_start, i64);
        impl_range!($range, $get_start, i128);
        impl_range!($range, $get_start, isize);
    };
}

fn range_start<Idx: Copy>(r: &Range<Idx>) -> Idx {
    r.start
}

fn range_from_start<Idx: Copy>(r: &RangeFrom<Idx>) -> Idx {
    r.start
}

fn range_inclusive_start<Idx: Copy>(r: &RangeInclusive<Idx>) -> Idx {
    *r.start()
}

impl_range_all_nums!(Range, range_start);
impl_range_all_nums!(RangeFrom, range_from_start);
impl_range_all_nums!(RangeInclusive, range_inclusive_start);

pub struct LazyBuffer<I: Iterator> {
    iter: I,
    buffer: Vec<I::Item>,
}

default impl<I> IntoBuffer for I
where
    I: Iterator,
    I::Item: Clone,
{
    type Into = LazyBuffer<I>;
}

impl<I> Buffer<I> for LazyBuffer<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn from_source(source: I) -> Self {
        LazyBuffer {
            iter: source,
            buffer: Vec::new(),
        }
    }

    fn get(&mut self, index: usize) -> Option<Self::Item> {
        loop {
            if self.buffer.len() > index {
                return Some(self.buffer[index].clone());
            }

            match self.iter.next() {
                Some(item) => {
                    self.buffer.push(item);
                }
                None => {
                    return None;
                }
            }
        }
    }
}
