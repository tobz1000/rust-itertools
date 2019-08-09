/// An iterator to iterate through all the `k`-permutations of a series of items.
///
/// Source items are distinguished by their position, not value; so if there
/// are identical items in the source, there will be some identical permutation
/// iterations.
///
/// Can be constructed from an in-memory list of items directly; or from an
/// iterator, with the
/// [`.permuatations()`](../trait.Itertools.html#method.permutations) method.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Permutations<I: Iterator> {
    vals: Vec<I::Item>,
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
    },
    Empty
}

pub fn permutations<I: Iterator>(iter: I, k: usize) -> Permutations<I> {
    let vals: Vec<I::Item> = iter.collect();
    let state = PermutationState::new(vals.len(), k);

    Permutations { vals, state }
}

impl<I> Iterator for Permutations<I>
where
    I: Iterator,
    I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Permutations { ref vals, ref mut state } = self;

        let perm_indices = state.next()?;
        let perm = perm_indices.into_iter().map(|&p| vals[p].clone()).collect();

        Some(perm)
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
        if k > n {
            PermutationState::Empty
        } else {
            PermutationState::Stopped { n: n, k: k }
        }
    }

    fn advance(&mut self) {
        *self = match self {
            &mut PermutationState::Stopped { n, k } => {
                PermutationState::Ongoing {
                    indices: (0..n).collect(),
                    cycles: (n - k..n).rev().collect(),
                }
            },
            &mut PermutationState::Ongoing { ref mut cycles, ref mut indices } => {
                let n = indices.len();
                let k = cycles.len();

                for i in (0..k).rev() {
                    if cycles[i] == 0 {
                        cycles[i] = n - i - 1;

                        let to_push = indices.remove(i);
                        indices.push(to_push);
                    } else {
                        let swap_index = n - cycles[i];
                        indices.swap(i, swap_index);

                        cycles[i] -= 1;
                        return;
                    }
                }

                PermutationState::Stopped { n, k }
            },
            &mut PermutationState::Empty => PermutationState::Empty,
        }
    }

    fn next(&mut self) -> Option<&[usize]> {
        self.advance();

        match self {
            &mut PermutationState::Stopped { .. } => None,
            &mut PermutationState::Ongoing { ref indices, ref cycles } => {
                Some(&indices[0..cycles.len()])
            },
            &mut PermutationState::Empty => None
        }
    }

    fn remaining(&self) -> Option<usize> {
        match self {
            &PermutationState::Stopped { n, k } => {
                (n - k + 1..n + 1).fold(Some(1), |acc, i| {
                    acc.and_then(|acc| acc.checked_mul(i))
                })
            },
            &PermutationState::Ongoing { ref cycles, ref indices } => {
                let mut size: usize = 0;

                for (i, &c) in cycles.iter().enumerate() {
                    let radix = indices.len() - i;
                    let next_size = size.checked_mul(radix)
                        .and_then(|size| size.checked_add(c));

                    size = match next_size {
                        Some(size) => size,
                        None => { return None; }
                    };
                }

                Some(size)
            },
            &PermutationState::Empty => Some(0)
        }
    }
}
