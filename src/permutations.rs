// ```python
// def permutations(iterable, r=None):
//     # permutations('ABCD', 2) --> AB AC AD BA BC BD CA CB CD DA DB DC
//     # permutations(range(3)) --> 012 021 102 120 201 210
//     pool = tuple(iterable)
//     n = len(pool)
//     r = n if r is None else r
//     if r > n:
//         return
//     indices = range(n)
//     cycles = range(n, n-r, -1)
//     yield tuple(pool[i] for i in indices[:r])
//     while n:
//         for i in reversed(range(r)):
//             cycles[i] -= 1
//             if cycles[i] == 0:
//                 # Remove value at indices[i] and push it to end
//                 indices[i:] = indices[i+1:] + indices[i:i+1]
//                 cycles[i] = n - i
//             else:
//                 j = cycles[i]
//                 indices[i], indices[-j] = indices[-j], indices[i]
//                 yield tuple(pool[i] for i in indices[:r])
//                 break
//         else:
//             return
// ```

pub fn permutations(n: usize, k: usize) -> Permutations {
    Permutations::Stopped { n, k }
}

pub enum Permutations {
    Stopped {
        n: usize,
        k: usize
    },
    Ongoing {
        indices: Vec<usize>,
        cycles: Vec<usize>
    }
}

impl Permutations {
    fn advance(&mut self) {
        match self {
            &mut Permutations::Stopped { n, k } => {
                if n == 0 || k == 0 || k > n {
                    return;
                }

                *self = Permutations::Ongoing {
                    cycles: (n - k + 1..=n).rev().collect(),
                    indices: (0..n).collect()
                };
            },
            Permutations::Ongoing { cycles, indices } => {
                let i_len = indices.len();

                for i in (0..cycles.len()).rev() {
                    let c = &mut cycles[i];

                    *c -= 1;

                    if *c == 0 {
                        *c = i_len - i;

                        let to_push = indices.remove(i);
                        indices.push(to_push);
                    } else {
                        indices.swap(i, i_len - *c);
                        return;
                    }
                }

                *self = Permutations::Stopped {
                    n: indices.len(),
                    k: cycles.len()
                };
            }
        }
    }

    fn get(&mut self) -> Option<&[usize]> {
        match self {
            Permutations::Stopped { .. } => None,
            Permutations::Ongoing { indices, cycles } => {
                Some(&indices[0..cycles.len()])
            }
        }
    }

    pub fn stream(&mut self) -> Option<&[usize]> {
        self.advance();
        self.get()
    }
}

impl Iterator for Permutations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();
        self.get().map(|s| s.to_vec())
    }
}