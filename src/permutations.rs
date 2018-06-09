/// An iterator to iterate through all the `k`-permutations of a series of items.
///
/// Can be constructed from an in-memory list of items directly; or from an
/// iterator, with the
/// [`.permuatations()`](../trait.Itertools.html#method.permutations) method.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Permutations<S>
    where S: PermutationSource
{
    vals: S,
    state: PermutationState
}

enum PermutationState {
    Stopped {
        n: usize,
        k: usize
    },
    Ongoing {
        indices: Vec<usize>,
        cycles: Vec<usize>
    }
}

/// Functionality required to construct and iterate a
/// [`Permutatations`](struct.Permutations.html) from a
/// data source.
pub trait PermutationSource {
    /// The type to be yielded, within a `Vec`, for each permutation.
    type Item;

    /// Builds a permutation from the data source, given a list of input indexes.
    /// The length of the returned `Vec` should match the length of `perm`.
    fn perm_to_vec(&self, perm: &[usize]) -> Vec<Self::Item>;

    /// Returns he number of items within the data source to be permuted.
    fn len(&self) -> usize;
}

pub struct PermutationIndicesSource(usize);

impl Permutations<PermutationIndicesSource> {
    /// Creates a new `Permutation` over the range `0..n`, yielding permutations
    /// of length `k`.
    ///
    /// ```
    /// use itertools::Permutations;
    ///
    /// let perms = Permutations::new(3, 2);
    /// itertools::assert_equal(perms, vec![
    ///     vec![0, 1],
    ///     vec![0, 2],
    ///     vec![1, 0],
    ///     vec![1, 2],
    ///     vec![2, 0],
    ///     vec![2, 1],
    /// ]);
    /// ```
    pub fn new(n: usize, k: usize) -> Self {
        Permutations::from_vals(PermutationIndicesSource(n), k)
    }
}

impl<S> Permutations<S>
    where S: PermutationSource
{
    /// Creates a new `Permutation` over the provided data source.
    ///
    /// If `vals` is a `Vec` of clonable items, the yielded permutations will be
    /// clones of the source items.
    ///
    /// If `vals` is a slice, the yielded permutations will be of references to
    /// the original items.
    ///
    /// ```
    /// use itertools::Permutations;
    ///
    /// let vals = vec!['a', 'b', 'c'];
    ///
    /// let ref_perms = Permutations::from_vals(vals.as_slice(), 2);
    /// itertools::assert_equal(ref_perms, vec![
    ///     vec![&'a', &'b'],
    ///     vec![&'a', &'c'],
    ///     vec![&'b', &'a'],
    ///     vec![&'b', &'c'],
    ///     vec![&'c', &'a'],
    ///     vec![&'c', &'b'],
    /// ]);
    ///
    /// let owned_perms = Permutations::from_vals(vec!['a', 'b', 'c'], 2);
    /// itertools::assert_equal(owned_perms, vec![
    ///     vec!['a', 'b'],
    ///     vec!['a', 'c'],
    ///     vec!['b', 'a'],
    ///     vec!['b', 'c'],
    ///     vec!['c', 'a'],
    ///     vec!['c', 'b'],
    /// ]);
    /// ```
    pub fn from_vals(vals: S, k: usize) -> Self {
        let state = PermutationState::new(vals.len(), k);

        Permutations { vals, state }
    }
}

impl<S> Iterator for Permutations<S>
    where S: PermutationSource
{
    type Item = Vec<S::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let Permutations { vals, state } = self;
        state.next().map(|perm| {
            let next = vals.perm_to_vec(perm);
            assert_eq!(perm.len(), next.len(), "Permutation length incorrect");
            next
        })
    }

    fn count(self) -> usize {
        if let Some(count) = self.state.remaining() {
            count
        } else {
            panic!("Iterator count greater than usize::MAX");
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(size) = self.state.remaining() {
            (size, Some(size))
        } else {
            (::std::usize::MAX, None)
        }
    }
}

impl PermutationState {
    fn new(n: usize, k: usize) -> PermutationState {
        PermutationState::Stopped { n, k }
    }

    fn advance(&mut self) {
        match self {
            &mut PermutationState::Stopped { n, k } => {
                if n == 0 || k == 0 || k > n {
                    return;
                }

                *self = PermutationState::Ongoing {
                    indices: (0..n).collect(),
                    cycles: (n - k..n).rev().collect(),
                };
            },
            PermutationState::Ongoing { cycles, indices } => {
                for (i, c) in cycles.iter_mut().enumerate().rev() {
                    if *c == 0 {
                        *c = indices.len() - i - 1;

                        let to_push = indices.remove(i);
                        indices.push(to_push);
                    } else {
                        let swap_index = indices.len() - *c;
                        indices.swap(i, swap_index);

                        *c -= 1;
                        return;
                    }
                }

                *self = PermutationState::Stopped {
                    n: indices.len(),
                    k: cycles.len()
                };
            }
        }
    }

    fn next(&mut self) -> Option<&[usize]> {
        self.advance();

        match self {
            PermutationState::Stopped { .. } => None,
            PermutationState::Ongoing { indices, cycles } => {
                Some(&indices[0..cycles.len()])
            }
        }
    }

    fn remaining(&self) -> Option<usize> {
        match self {
            &PermutationState::Stopped { n, k } => {
                if n == 0 || k == 0 || k > n {
                    Some(0)
                } else {
                    (n - k + 1..=n).fold(Some(1), |acc, i| {
                        acc.and_then(|acc| acc.checked_mul(i))
                    })
                }
            },
            PermutationState::Ongoing { cycles, indices } => {
                cycles.iter()
                    .enumerate()
                    .fold(Some(0), |acc, (i, &c)| {
                        acc.and_then(|acc| {
                            let radix = indices.len() - i;
                            radix.checked_mul(c).and_then(|s| s.checked_add(acc))
                        })
                    })
            }
        }
    }
}

impl PermutationSource for PermutationIndicesSource {
    type Item = usize;

    fn perm_to_vec(&self, perm: &[usize]) -> Vec<usize> {
        perm.to_vec()
    }

    fn len(&self) -> usize {
        self.0
    }
}

impl<T> PermutationSource for Vec<T>
    where T: Clone
{
    type Item = T;

    fn perm_to_vec(&self, perm: &[usize]) -> Vec<T> {
        perm.into_iter().map(|&p| self[p].clone()).collect()
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<'a, T: 'a> PermutationSource for &'a [T] {
    type Item = &'a T;

    fn perm_to_vec(&self, perm: &[usize]) -> Vec<&'a T> {
        perm.into_iter().map(|&p| &self[p]).collect()
    }

    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}
