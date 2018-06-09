use std::iter::Product;

#[derive(Debug)]
pub struct Permutations<S> {
    vals: S,
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

pub trait PermutationSource {
    type Item;

    fn perm_to_vec(&self, perm: &[usize]) -> Vec<Self::Item>;

    fn len(&self) -> usize;
}

#[derive(Debug)]
pub struct PermutationIndicesSource(usize);

impl Permutations<PermutationIndicesSource> {
    pub fn new(n: usize, k: usize) -> Self {
        Permutations::from_vals(PermutationIndicesSource(n), k)
    }
}

impl<S> Permutations<S>
    where S: PermutationSource
{
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
        state.stream().map(|perm| vals.perm_to_vec(perm))
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
