use std::fmt;
use std::iter::once;

use super::lazy_buffer::LazyBuffer;

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
    vals: LazyBuffer<I>,
    state: PermutationState,
}

enum PermutationState {
    StartUnknownLen {
        k: usize,
    },
    OngoingUnknownLen {
        k: usize,
    },
    Complete(CompleteState),
    Empty,
}

enum CompleteState {
    Start {
        n: usize,
        k: usize,
    },
    Ongoing {
        indices: Vec<usize>,
        cycles: Vec<usize>,
    }
}

enum CompleteStateRemaining {
    Known(usize),
    Overflow,
}

pub fn permutations<I: Iterator>(iter: I, k: usize) -> Permutations<I> {
    let mut vals = LazyBuffer::new(iter);

    if k == 0 {
        // Special case, yields single empty vec; `n` is irrelevant
        let state = PermutationState::Complete(CompleteState::Start { n: 0, k: 0 });

        return Permutations {
            vals,
            state
        };
    }

    let mut enough_vals = true;

    while vals.len() < k {
        if !vals.get_next() {
            enough_vals = false;
            break;
        }
    }

    let state = if enough_vals {
        PermutationState::StartUnknownLen { k }
    } else {
        PermutationState::Empty
    };

    Permutations {
        vals,
        state
    }
}

impl<I> Iterator for Permutations<I>
where
    I: Iterator,
    I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();

        let &mut Permutations { ref vals, ref state } = self;

        match state {
            &PermutationState::StartUnknownLen { .. } => panic!("unexpected iterator state"),
            &PermutationState::OngoingUnknownLen { k } => {
                let latest_idx = vals.len() - 1;

                let indices = (0..(k - 1)).chain(once(latest_idx));

                Some(indices.map(|i| vals[i].clone()).collect())
            }
            &PermutationState::Complete(CompleteState::Start { .. }) => None,
            &PermutationState::Complete(CompleteState::Ongoing { ref indices, ref cycles }) => {
                let k = cycles.len();

                Some(indices[0..k].iter().map(|&i| vals[i].clone()).collect())
            },
            &PermutationState::Empty => None
        }
    }

    fn count(self) -> usize {
        let Permutations { vals, state } = self;

        fn from_complete(complete_state: CompleteState) -> usize {
            match complete_state.remaining() {
                CompleteStateRemaining::Known(count) => count,
                CompleteStateRemaining::Overflow => {
                    panic!("Iterator count greater than usize::MAX");
                }
            }
        }

        match state {
            PermutationState::StartUnknownLen { k } |
            PermutationState::OngoingUnknownLen { k } => {
                let n = vals.len() + vals.it.count();
                let complete_state = CompleteState::Start { n, k };

                from_complete(complete_state)
            },
            PermutationState::Complete(state) => from_complete(state),
            PermutationState::Empty => 0
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            PermutationState::StartUnknownLen { .. } |
            PermutationState::OngoingUnknownLen { .. } => (0, None), // TODO can we improve this lower bound?
            PermutationState::Complete(ref state) => match state.remaining() {
                CompleteStateRemaining::Known(count) => (count, Some(count)),
                CompleteStateRemaining::Overflow => (::std::usize::MAX, None)
            }
            PermutationState::Empty => (0, Some(0))
        }
    }
}

impl<I> Permutations<I>
where
    I: Iterator,
    I::Item: Clone
{
    fn advance(&mut self) {
        let &mut Permutations { ref mut vals, ref mut state } = self;

        *state = match state {
            &mut PermutationState::StartUnknownLen { k } => {
                PermutationState::OngoingUnknownLen { k }
            }
            &mut PermutationState::OngoingUnknownLen { k } => {
                if vals.get_next() {
                    return;
                } else {
                    let n = vals.len();
                    let prev_iteration_count = n - k + 1;
                    let mut complete_state = CompleteState::Start { n, k };

                    println!("Advancing {}x; n={} k={}", prev_iteration_count, n, k);

                    // Advance the complete-state iterator to the correct point
                    for _ in 0..(prev_iteration_count + 1) {
                        complete_state.advance();
                    }

                    PermutationState::Complete(complete_state)
                }
            }
            &mut PermutationState::Complete(ref mut state) => {
                state.advance();

                return;
            }
            &mut PermutationState::Empty => { return; }
        };
    }
}

impl CompleteState {
    fn advance(&mut self) {
        *self = match self {
            &mut CompleteState::Start { n, k } => {
                let indices = (0..n).collect();
                let cycles = ((n - k)..n).rev().collect();

                CompleteState::Ongoing {
                    cycles,
                    indices
                }
            },
            &mut CompleteState::Ongoing { ref mut indices, ref mut cycles } => {
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

                CompleteState::Start { n, k }
            }
        }
    }

    fn remaining(&self) -> CompleteStateRemaining {
        use self::CompleteStateRemaining::{Known, Overflow};

        match self {
            &CompleteState::Start { n, k } => {
                let count: Option<usize> = (n - k + 1..n + 1).fold(Some(1), |acc, i| {
                    acc.and_then(|acc| acc.checked_mul(i))
                });

                match count {
                    Some(count) => Known(count),
                    None => Overflow
                }
            }
            &CompleteState::Ongoing { ref indices, ref cycles } => {
                let mut count: usize = 0;

                for (i, &c) in cycles.iter().enumerate() {
                    let radix = indices.len() - i;
                    let next_count = count.checked_mul(radix)
                        .and_then(|count| count.checked_add(c));

                    count = match next_count {
                        Some(count) => count,
                        None => { return Overflow; }
                    };
                }

                Known(count)
            }
        }
    }
}