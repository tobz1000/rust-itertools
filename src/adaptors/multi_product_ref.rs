#![cfg(feature = "use_std")]

use Itertools;

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `Vec<I>`.
///
/// See [`.multi_cartesian_product()`](../trait.Itertools.html#method.multi_cartesian_product)
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProductRef<I>
    where I: Iterator + Clone
{
    iters: Vec<MultiProductIter<I>>,
    cur: Option<Vec<I::Item>>,
}


/// Create a new cartesian product iterator over an arbitrary number
/// of iterators of the same type.
///
/// Iterator element is of type `Vec<H::Item::Item>`.
pub fn multi_cartesian_product_ref<H>(iters: H)
    -> MultiProductRef<<H::Item as IntoIterator>::IntoIter>
    where H: Iterator,
          H::Item: IntoIterator,
          <H::Item as IntoIterator>::IntoIter: Clone,
{
    MultiProductRef {
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

impl<I> MultiProductRef<I>
    where I: Iterator + Clone
{
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

            // If .next() returns None twice, iterator is empty, therefore whole
            // product is empty; finish..
            last.iter.next()?
        };

        Some(())
    }

    pub fn next(&mut self) -> Option<&[I::Item]> {
        match self.cur {
            None => {
                self.cur = Self::initial_iteration(&mut self.iters);
            },
            Some(ref mut cur) => {
                Self::iterate_last(&mut self.iters, cur)?;
            }
        }

        if let Some(ref cur) = self.cur {
            Some(cur)
        } else {
            None
        }
    }

}