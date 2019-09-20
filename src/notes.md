# `BufferSource`

Trait to allow direct use of underlying buffer for `std::slice::Iter` and `std::vec::IntoIter`, rather than iterating, for adaptors which require some level of non-sequential access to their input iterator elements.

Could also apply to deterministic getter-functions, e.g. for `std::ops::Range`.

Not necessary/suitable for types which:
    - Use input elements repeatedly, but sequentially (cloning the iterator on each cycle is more appropriate)
    - Require the elements to be collected into a different data structure (e.g. a HashSet)

## Adaptors to use `BufferSource`

- `permutations`
- `combinations`
- `combinations_with_replacement`
- `multi_cartesion_product`
  - For the meta-iterator only; not for sub-iterators. While sub-iterators do repeat their elements, the elements are used sequentially, so re-cloning the sub-iterators each time is more appropriate.
- `multipeek`?
- `rev`?
    - Might be covered by `DoubleEndedIterator`
- `tuple_combinations`