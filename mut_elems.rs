#![doc(html_root_url = "https://docs.rs/mut-elems/0.2.0")]

/*!

Get simultaneous mutable access to multiple elements of a
mutable array, slice or `Vec`. This is a generalization of
[slice::split_at_mut] to individual elements rather
than just a pair of subslices.

# Examples

```
use mut_elems::*;

let mut a = [1u8, 2, 3, 4];

let es: [&mut u8; 2] = a.mut_elems(&[1, 3]).unwrap();
*es[0] = 5;
*es[1] = 7;
assert_eq!([1, 5, 3, 7], a);

let es: [&mut u8; 4] = a.as_mut_elems();
*es[1] = 5;
*es[3] = 7;
assert_eq!([1, 5, 3, 7], a);

let mut aref: &mut [u8] = a.as_mut();
let mut es: Vec<&mut u8> = aref.as_mut_elems_vec();
*es[1] = 5;
*es[3] = 7;
assert_eq!([1, 5, 3, 7], a);
```

*/

use thiserror::Error;

/// Failure cases for [MutElemsExt::mut_elems].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutElemsError {
    /// There is a repeated index in the provided indices.
    #[error("indices {first} and {second} are both {index}")]
    IndicesOverlap {
        /// First position of repeated index in indices.
        first: usize,
        /// Second position of repeated index in indices.
        second: usize,
        /// Value of repeated index.
        index: usize,
    },
    /// A provided index is out of bounds.
    #[error("index {position} is {index}, but target length is {length}")]
    IndexBound {
        /// Position of out-of-bounds index in indices.
        position: usize,
        /// Value out-of-bounds index.
        index: usize,
        /// Number of elements in target: should be greater than index.
        length: usize,
    },
}
use MutElemsError::*;

pub trait MutElemsExt<T> {
    /// Return mutable references to elements of `self`
    /// at each of the index positions given by `indices`.
    ///
    /// All indices must be unique, as Rust does not allow
    /// multiple mutable references to the same object.
    ///
    /// # Errors
    ///
    /// Will return an error if any of the indices are out of bounds,
    /// or if any pair of indices is identical.
    fn mut_elems<'a, const N: usize>(
        &'a mut self,
        indices: &[usize; N],
    ) -> Result<[&'a mut T; N], MutElemsError>;
}

pub trait AsMutElemsExt<const N: usize, T> {
    /// Return an array of mutable references to each
    /// of the elements of the input array.
    fn as_mut_elems(&mut self) -> [&mut T; N];
}

pub trait AsMutElemsVecExt<T> {
    /// Return a `Vec` of mutable references to each
    /// of the elements of the input `Vec`.
    fn as_mut_elems_vec(&mut self) -> Vec<&mut T>;
}

impl<T> MutElemsExt<T> for [T] {
    fn mut_elems<'a, const N: usize>(
        &'a mut self,
        indices: &[usize; N],
    ) -> Result<[&'a mut T; N], MutElemsError> {
        // Index checking. 0, 1, 2 are special-cased for
        // performance, in particular since 2 may be commonly
        // used.
        match N {
            0 | 1 => (),
            2 => {
                if indices[0] == indices[1] {
                    return Err(IndicesOverlap {
                        first: indices[0],
                        second: indices[1],
                        index: indices[0],
                    });
                }
            }
            _ => {
                use std::collections::HashMap;

                let mut seen: HashMap<usize, usize> = HashMap::with_capacity(indices.len());

                for (i, ix) in indices.iter().enumerate() {
                    if seen.contains_key(ix) {
                        let j = seen[ix];
                        return Err(IndicesOverlap {
                            first: j,
                            second: i,
                            index: *ix,
                        });
                    }
                    seen.insert(*ix, i);
                }
            }
        }

        // Index bounds checking.
        let nself = self.len();
        for (i, ix) in indices.iter().enumerate() {
            if *ix >= nself {
                return Err(IndexBound {
                    position: i,
                    index: *ix,
                    length: nself,
                });
            }
        }

        // Safety: Indices have been checked for inequality, so
        // they must indicate unique locations.  Bounds checking
        // has already been done, so we can bypass checking the
        // indices.  `from_fn()` guarantees that `i` is
        // in-bounds, so we can bypass checking that.
        Ok(std::array::from_fn(|i| unsafe {
            &mut *(self.get_unchecked_mut(*indices.get_unchecked(i)) as *mut T)
        }))
    }
}

impl<const N: usize, T> AsMutElemsExt<N, T> for [T; N] {
    fn as_mut_elems(&mut self) -> [&mut T; N] {
        // Safety: `from_fn()` guarantees that indices `i`
        // are in-bounds and unique.
        std::array::from_fn(|i| unsafe { &mut *(self.get_unchecked_mut(i) as *mut T) })
    }
}

impl<T, V> AsMutElemsVecExt<T> for V where V: AsMut<[T]> {
    fn as_mut_elems_vec(&mut self) -> Vec<&mut T> {
        // Safety: iteration guarantees that elements
        // are in-bounds and unique.
        self.as_mut().iter_mut()
            .map(|r| unsafe { &mut *(r as *mut T) })
            .collect()
    }
}

#[test]
fn test_mut_elems() {
    let mut test_array = [1u8, 2, 3, 4];

    assert_eq!([&1], test_array.mut_elems(&[0]).unwrap());
    assert_eq!([&4], test_array.mut_elems(&[3]).unwrap());
    assert_eq!([&2, &3], test_array.mut_elems(&[1, 2]).unwrap());
    assert_eq!([&1, &3, &4], test_array.mut_elems(&[0, 2, 3]).unwrap());

    match test_array.mut_elems(&[4]) {
        Err(MutElemsError::IndexBound {
            position,
            index,
            length,
        }) => {
            assert_eq!(position, 0);
            assert_eq!(index, 4);
            assert_eq!(length, 4);
        }
        _ => panic!(),
    }

    match test_array.mut_elems(&[1, 2, 1]) {
        Err(MutElemsError::IndicesOverlap {
            first,
            second,
            index,
        }) => {
            assert_eq!(first, 0);
            assert_eq!(second, 2);
            assert_eq!(index, 1);
        }
        _ => panic!(),
    }

    let es = test_array.mut_elems(&[1, 3]).unwrap();
    *es[0] = 5;
    *es[1] = 7;
    assert_eq!([1, 5, 3, 7], test_array);
}

#[test]
fn test_as_mut_elems() {
    let mut test_array = [1u8, 2, 3, 4];
    let es = test_array.as_mut_elems();
    assert_eq!([&1, &2, &3, &4], es);
    *es[1] = 5;
    *es[3] = 7;
    assert_eq!([1, 5, 3, 7], test_array);
}

#[test]
fn test_as_mut_elems_vec() {
    let mut test_vec = vec![1u8, 2, 3, 4];
    let mut es = test_vec.as_mut_elems_vec();
    assert_eq!(vec![&1, &2, &3, &4], es);
    *es[1] = 5;
    *es[3] = 7;
    assert_eq!(vec![1, 5, 3, 7], test_vec);
}
