#![cfg(feature = "use_std")]

use size_hint;
use Itertools;
use streaming_iterator::StreamingIterator;
use std::marker::PhantomData;

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `Vec<I::Item>`.
///
/// See [`.multi_cartesian_product()`](../trait.Itertools.html#method.multi_cartesian_product)
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProduct<I>
    where I: Iterator + Clone
{
    iters: Vec<MultiProductIter<I>>,
    cur: Option<Vec<I::Item>>,
}

pub struct MultiProductStreaming<I>(MultiProduct<I>)
    where I: Iterator + Clone;

/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `[I::Item; N]`, where `N` is the number of
/// sub-iterators.
///
/// Type `A` is a dummy array type, the length of which is used to determine the
/// length of yielded items when iterating. The array item component of `A` is
/// not used.
///
/// See [`iproduct_arr`](../macro.iproduct_arr.html) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProductArray<I, A>(MultiProduct<I>, PhantomData<A>)
    where I: Iterator + Clone;

/// Create a new cartesian product iterator over an arbitrary number
/// of iterators of the same type.
///
/// Iterator element is of type `Vec<H::Item::Item>`.
pub fn multi_cartesian_product<H>(iters: H)
    -> MultiProduct<<H::Item as IntoIterator>::IntoIter>
    where H: Iterator,
          H::Item: IntoIterator,
          <H::Item as IntoIterator>::IntoIter: Clone,
{
    MultiProduct {
        iters: iters.map(|iter| {
            let iter = iter.into_iter();
            MultiProductIter {
                iter: iter.clone(),
                iter_orig: iter
            }
        }).collect(),
        cur: None
    }
}

#[derive(Clone, Debug)]
/// Holds the state of a single iterator within a MultiProduct.
struct MultiProductIter<I>
    where I: Iterator + Clone
{
    iter: I,
    iter_orig: I,
}

impl<I> MultiProduct<I>
    where I: Iterator + Clone
{
    pub fn streaming(self) -> MultiProductStreaming<I> {
        MultiProductStreaming(self)
    }

    pub fn array<A>(self) -> MultiProductArray<I, A> {
        MultiProductArray(self, PhantomData::<A>)
    }

    /// Returns first item of each iterator as a `Vec`, or None if any iterator
    /// is empty.
    fn initial_iteration(
        multi_iters: &mut [MultiProductIter<I>]
    ) -> Option<Vec<I::Item>> {
        let iter_count = multi_iters.len();

        let initial: Vec<I::Item> = multi_iters.iter_mut()
            .map(|multi_iter| multi_iter.iter.next())
            .while_some()
            .collect();

        if initial.len() == iter_count {
            Some(initial)
        } else {
            None
        }
    }

    /// Iterates the rightmost iterator, then recursively iterates iterators
    /// to the left if necessary.
    ///
    /// Returns Some(()) if the iteration succeeded, else None.
    fn iterate_last(
        multi_iters: &mut [MultiProductIter<I>],
        curs: &mut [I::Item]
    ) -> Option<()> {
        // If split fails, reached end of iterator list; all iterators finished.
        let (last, rest) = multi_iters.split_last_mut()?;

        // Should be the same length as multi_iters
        let (last_cur, rest_curs) = curs.split_last_mut().unwrap();

        *last_cur = if let Some(next) = last.iter.next() {
            next
        } else {
            last.iter = last.iter_orig.clone();

            // Propagate failures from further multi_iters
            Self::iterate_last(rest, rest_curs)?;

            // If restarted iter returns None, it is empty, therefore whole
            // product is empty; finish.
            last.iter.next()?
        };

        Some(())
    }

    fn in_progress(&self) -> bool {
        self.cur.is_some()
    }

    fn advance(&mut self) {
        if self.iters.len() == 0 {
            return;
        }

        let mut finished = false;

        match self.cur {
            None => {
                self.cur = Self::initial_iteration(&mut self.iters);
            },
            Some(ref mut cur) => {
                finished = Self::iterate_last(&mut self.iters, cur) == None;
            }
        }

        if finished {
            self.cur = None;
        }
    }

    fn _count(self) -> usize {
        if self.iters.len() == 0 {
            return 0;
        }

        if !self.in_progress() {
            return self.iters.into_iter().fold(1, |acc, multi_iter| {
                acc * multi_iter.iter.count()
            });
        }

        self.iters.into_iter().fold(
            0,
            |acc, MultiProductIter { iter, iter_orig }| {
                let total_count = iter_orig.count();
                let cur_count = iter.count();
                acc * total_count + cur_count
            }
        )
    }

    fn _size_hint(&self) -> (usize, Option<usize>) {
        // Not ExactSizeIterator because size may be larger than usize
        if self.iters.len() == 0 {
            return (0, Some(0));
        }

        if !self.in_progress() {
            return self.iters.iter().fold((1, Some(1)), |acc, multi_iter| {
                size_hint::mul(acc, multi_iter.iter.size_hint())
            });
        }

        self.iters.iter().fold(
            (0, Some(0)),
            |acc, &MultiProductIter { ref iter, ref iter_orig }| {
                let cur_size = iter.size_hint();
                let total_size = iter_orig.size_hint();
                size_hint::add(size_hint::mul(acc, total_size), cur_size)
            }
        )
    }
}

impl<I> Iterator for MultiProduct<I>
    where I: Iterator + Clone,
          I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();

        if let Some(ref cur) = self.cur {
            Some(cur.clone())
        } else {
            None
        }
    }

    fn count(self) -> usize {
        self._count()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self._size_hint()
    }

    fn last(self) -> Option<Self::Item> {
        let iter_count = self.iters.len();

        let lasts: Vec<I::Item> = self.iters.into_iter()
            .map(|multi_iter| multi_iter.iter.last())
            .while_some()
            .collect();

        if lasts.len() == iter_count {
            Some(lasts)
        } else {
            None
        }
    }
}

impl<I> StreamingIterator for MultiProductStreaming<I>
    where I: Iterator + Clone
{
    type Item = [I::Item];

    fn advance(&mut self) {
        self.0.advance()
    }

    fn get(&self) -> Option<&Self::Item> {
        if let Some(ref cur) = self.0.cur {
            Some(cur)
        } else {
            None
        }
    }

    fn count(self) -> usize {
        self.0._count()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0._size_hint()
    }
}

macro_rules! multi_product_array_impl {
    ($N:expr, $($M:expr,)*) => {
        multi_product_array_impl!($($M,)*);

        impl<I, _A> Iterator for MultiProductArray<I, [_A; $N]>
            where I: Iterator + Clone,
                  I::Item: Clone
        {
            type Item = [I::Item; $N];

            fn next(&mut self) -> Option<Self::Item> {
                self.0.advance();

                if let Some(ref cur) = self.0.cur {
                    let ptr = cur.as_ptr() as *const Self::Item;
                    let arr_ref = unsafe { &*ptr };
                    Some(arr_ref.clone())
                } else {
                    None
                }
            }

            fn count(self) -> usize {
                self.0._count()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0._size_hint()
            }

            fn last(self) -> Option<Self::Item> {
                let mut _lasts = self.0.iters.into_iter()
                    .map(|multi_iter| multi_iter.iter.last())
                    .while_some();

                Some([ $({ $M; _lasts.next()? },)* ])
            }
        }
    };
    () => {};
}

multi_product_array_impl!{
    32, 31, 30,
    29, 28, 27, 26, 25, 24, 23, 22, 21, 20,
    19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
    9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
}