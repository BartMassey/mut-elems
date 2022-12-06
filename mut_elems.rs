use thiserror::Error;

/// Failure cases for [[mut_elems]].
#[derive(Error, Debug)]
pub enum MutElemsError {
    #[error("indices {first} and {second} are both {value}")]
    IndicesOverlap {
        first: usize,
        second: usize,
        value: usize,
    },
    #[error("index {position} is {value}, but target length is {length}")]
    IndexBound {
        position: usize,
        value: usize,
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
    fn as_mut_elems(&mut self) -> Vec<&mut T>;
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
                        value: indices[0],
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
                            value: *ix,
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
                    value: *ix,
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
        std::array::from_fn(|i| unsafe {
            &mut *(self.get_unchecked_mut(i) as *mut T)
        })
    }
}

impl<T> AsMutElemsVecExt<T> for Vec<T> {
    fn as_mut_elems(&mut self) -> Vec<&mut T> {
        // Safety: iteration guarantees that elements
        // are in-bounds and unique.
        self
            .iter_mut()
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
    assert_eq!(
        [&1, &3, &4],
        test_array.mut_elems(&[0, 2, 3]).unwrap()
    );

    match test_array.mut_elems(&[4]) {
        Err(MutElemsError::IndexBound {
            position,
            value,
            length,
        }) => {
            assert_eq!(position, 0);
            assert_eq!(value, 4);
            assert_eq!(length, 4);
        }
        _ => panic!(),
    }

    match test_array.mut_elems(&[1, 2, 1]) {
        Err(MutElemsError::IndicesOverlap {
            first,
            second,
            value,
        }) => {
            assert_eq!(first, 0);
            assert_eq!(second, 2);
            assert_eq!(value, 1);
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
    let mut es = test_vec.as_mut_elems();
    assert_eq!(vec![&1, &2, &3, &4], es);
    *es[1] = 5;
    *es[3] = 7;
    assert_eq!(vec![1, 5, 3, 7], test_vec);
}
