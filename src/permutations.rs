use std::fmt;
use std::iter::once;

use super::lazy_buffer::LazyBuffer;

use crate::buffer::{Buffer, IntoBuffer};

/// An iterator adaptor that iterates through all the `k`-permutations of the
/// elements from an iterator.
///
/// See [`.permutations()`](../trait.Itertools.html#method.permutations) for
/// more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Permutations<I>
where
    I: Iterator + IntoBuffer,
    I::Item: Clone,
{
    vals: I::Into,
    state: PermutationState,
}

#[derive(Debug)]
enum PermutationState {
    StartUnknownLen { k: usize },
    OngoingUnknownLen { k: usize, min_n: usize },
    Complete(CompleteState),
    Empty,
}

#[derive(Debug)]
enum CompleteState {
    Start {
        n: usize,
        k: usize,
    },
    Ongoing {
        indices: Vec<usize>,
        cycles: Vec<usize>,
    },
}

enum CompleteStateRemaining {
    Known(usize),
    Overflow,
}

// impl<I> fmt::Debug for Permutations<I>
// where
//     I: Iterator + fmt::Debug,
//     I::Item: fmt::Debug,
// {
//     debug_fmt_fields!(Permutations, vals, state);
// }

pub fn permutations<I>(iter: I, k: usize) -> Permutations<I>
where
    I: Iterator + IntoBuffer,
    I::Item: Clone,
{
    let mut vals = iter.into_buffer();

    if k == 0 {
        // Special case, yields single empty vec; `n` is irrelevant
        let state = PermutationState::Complete(CompleteState::Start { n: 0, k: 0 });

        return Permutations { vals, state };
    }

    let state = if vals.get(k - 1).is_some() {
        PermutationState::StartUnknownLen { k }
    } else {
        PermutationState::Empty
    };

    Permutations { vals, state }
}

impl<I> Iterator for Permutations<I>
where
    I: Iterator + IntoBuffer,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();

        let Permutations { vals, state } = self;

        match state {
            &mut PermutationState::StartUnknownLen { .. } => panic!("unexpected iterator state"),
            &mut PermutationState::OngoingUnknownLen { k, min_n } => {
                let latest_idx = min_n - 1;
                let indices = (0..(k - 1)).chain(once(latest_idx));

                Some(indices.map(|i| vals.get(i).unwrap()).collect())
            }
            &mut PermutationState::Complete(CompleteState::Start { .. }) => None,
            &mut PermutationState::Complete(CompleteState::Ongoing {
                ref indices,
                ref cycles,
            }) => {
                let k = cycles.len();

                Some(
                    indices[0..k]
                        .iter()
                        .map(|&i| vals.get(i).unwrap())
                        .collect(),
                )
            }
            &mut PermutationState::Empty => None,
        }
    }

    // fn count(self) -> usize {
    //     let Permutations { vals, state } = self;

    //     fn from_complete(complete_state: CompleteState) -> usize {
    //         match complete_state.remaining() {
    //             CompleteStateRemaining::Known(count) => count,
    //             CompleteStateRemaining::Overflow => {
    //                 panic!("Iterator count greater than usize::MAX");
    //             }
    //         }
    //     }

    //     match state {
    //         PermutationState::StartUnknownLen { k } => {
    //             let n = vals.len() + vals.it.count();
    //             let complete_state = CompleteState::Start { n, k };

    //             from_complete(complete_state)
    //         }
    //         PermutationState::OngoingUnknownLen { k, min_n } => {
    //             let prev_iteration_count = min_n - k + 1;
    //             let n = vals.len() + vals.it.count();
    //             let complete_state = CompleteState::Start { n, k };

    //             from_complete(complete_state) - prev_iteration_count
    //         }
    //         PermutationState::Complete(state) => from_complete(state),
    //         PermutationState::Empty => 0,
    //     }
    // }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            PermutationState::StartUnknownLen { .. }
            | PermutationState::OngoingUnknownLen { .. } => (0, None), // TODO can we improve this lower bound?
            PermutationState::Complete(ref state) => match state.remaining() {
                CompleteStateRemaining::Known(count) => (count, Some(count)),
                CompleteStateRemaining::Overflow => (::std::usize::MAX, None),
            },
            PermutationState::Empty => (0, Some(0)),
        }
    }
}

impl<I> Permutations<I>
where
    I: Iterator + IntoBuffer,
    I::Item: Clone,
{
    fn advance(&mut self) {
        let Permutations { vals, state } = self;

        *state = match state {
            &mut PermutationState::StartUnknownLen { k } => {
                PermutationState::OngoingUnknownLen { k, min_n: k }
            }
            &mut PermutationState::OngoingUnknownLen { k, min_n } => {
                if vals.get(min_n).is_some() {
                    PermutationState::OngoingUnknownLen {
                        k,
                        min_n: min_n + 1,
                    }
                } else {
                    let n = min_n;
                    let prev_iteration_count = n - k + 1;
                    let mut complete_state = CompleteState::Start { n, k };

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
            &mut PermutationState::Empty => {
                return;
            }
        };
    }
}

impl CompleteState {
    fn advance(&mut self) {
        *self = match self {
            &mut CompleteState::Start { n, k } => {
                let indices = (0..n).collect();
                let cycles = ((n - k)..n).rev().collect();

                CompleteState::Ongoing { cycles, indices }
            }
            &mut CompleteState::Ongoing {
                ref mut indices,
                ref mut cycles,
            } => {
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
                if n < k {
                    return Known(0);
                }

                let count: Option<usize> = (n - k + 1..n + 1)
                    .fold(Some(1), |acc, i| acc.and_then(|acc| acc.checked_mul(i)));

                match count {
                    Some(count) => Known(count),
                    None => Overflow,
                }
            }
            &CompleteState::Ongoing {
                ref indices,
                ref cycles,
            } => {
                let mut count: usize = 0;

                for (i, &c) in cycles.iter().enumerate() {
                    let radix = indices.len() - i;
                    let next_count = count
                        .checked_mul(radix)
                        .and_then(|count| count.checked_add(c));

                    count = match next_count {
                        Some(count) => count,
                        None => {
                            return Overflow;
                        }
                    };
                }

                Known(count)
            }
        }
    }
}
