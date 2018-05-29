#![cfg(feature = "use_std")]

use size_hint;
use Itertools;
use streaming_iterator::StreamingIterator;

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `Vec<I>`.
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
    /// Returns true if the iteration succeeded, else false.
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
            // Propagate failures from further multi_iters
            Self::iterate_last(rest, rest_curs)?;

            last.iter = last.iter_orig.clone();

            // If restarted iter returns None, it is empty, therefore whole
            // product is empty; finish.
            last.iter.next()?
        };

        Some(())
    }

    fn in_progress(&self) -> bool {
        self.cur.is_none()
    }

    fn advance(&mut self) {
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

    fn _last(self) -> Option<Vec<I::Item>> {
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
        self._last()
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