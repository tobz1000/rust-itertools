use std::iter::Product;

#[derive(Debug)]
pub struct PermutationsVec<T> {
    vals: Vec<T>,
    state: PermutationState
}

#[derive(Debug)]
pub struct PermutationsRef<'a, T: 'a> {
    vals: &'a [T],
    state: PermutationState
}

#[derive(Debug)]
pub struct PermutationIndices{
    state: PermutationState
}

#[derive(Debug)]
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

impl<T> PermutationsVec<T> {
    pub fn new(vals: Vec<T>, k: usize) -> PermutationsVec<T> {
        let state = PermutationState::new(vals.len(), k);

        PermutationsVec { vals, state }
    }
}

impl<T> Iterator for PermutationsVec<T>
    where T: Clone
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let PermutationsVec { vals, state } = self;

        state.stream().map(|perm| {
            perm.into_iter().map(|&p| vals[p].clone()).collect()
        })
    }

    fn count(self) -> usize {
        self.state.size()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.state.size();
        (size, Some(size))
    }
}

impl<'a, T: 'a> PermutationsRef<'a, T> {
    pub fn new(vals: &'a [T], k: usize) -> PermutationsRef<'a, T> {
        let state = PermutationState::new(vals.len(), k);

        PermutationsRef { vals, state }
    }
}

impl<'a, T: 'a> Iterator for PermutationsRef<'a, T>
{
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let PermutationsRef { vals, state } = self;

        state.stream().map(|perm| {
            perm.into_iter().map(|&p| &vals[p]).collect()
        })
    }

    fn count(self) -> usize {
        self.state.size()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.state.size();
        (size, Some(size))
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

    fn stream(&mut self) -> Option<&[usize]> {
        self.advance();

        match self {
            PermutationState::Stopped { .. } => None,
            PermutationState::Ongoing { indices, cycles } => {
                Some(&indices[0..cycles.len()])
            }
        }
    }

    fn size(&self) -> usize {
        match self {
            &PermutationState::Stopped { n, k } => {
                Product::product(n - k + 1..=n)
            },
            PermutationState::Ongoing { cycles, .. } => {
                cycles.iter()
                    .rev()
                    .enumerate()
                    .fold(0, |acc, (i, &c)| acc + i * c)
            }
        }
    }
}

impl PermutationIndices {
    pub fn new(n: usize, k: usize) -> PermutationIndices {
        PermutationIndices { state: PermutationState::new(n, k) }
    }

    pub fn stream(&mut self) -> Option<&[usize]> {
        self.state.stream()
    }
}

impl Iterator for PermutationIndices {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.stream().map(|perm| perm.to_vec())
    }

    fn count(self) -> usize {
        self.state.size()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.state.size();
        (size, Some(size))
    }
}
