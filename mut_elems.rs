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

/// Return mutable references to elements of `target`
/// at each of the index positions given by `indices`.
///
/// All indices must be unique, as Rust does not allow
/// multiple mutable references to the same object.
pub fn mut_elems<'a, const N: usize, T>(
    target: &'a mut [T],
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
    let ntarget = target.len();
    for (i, ix) in indices.iter().enumerate() {
        if *ix >= ntarget {
            return Err(IndexBound {
                position: i,
                value: *ix,
                length: ntarget,
            });
        }
    }

    // Safety: Indices have been checked for inequality, so
    // they must indicate unique locations.  Bounds checking
    // has already been done, so we can bypass checking the
    // indices.  `from_fn()` guarantees that `i` is
    // in-bounds, so we can bypass checking that.
    Ok(std::array::from_fn(|i| unsafe {
        &mut *(target.get_unchecked_mut(*indices.get_unchecked(i)) as *mut T)
    }))
}

#[test]
fn test_mut_elems() {
    let mut test_array = [1u8, 2, 3, 4];

    assert_eq!([&1], mut_elems(&mut test_array, &[0]).unwrap());
    assert_eq!([&4], mut_elems(&mut test_array, &[3]).unwrap());
    assert_eq!([&2, &3], mut_elems(&mut test_array, &[1, 2]).unwrap());
    assert_eq!(
        [&1, &3, &4],
        mut_elems(&mut test_array, &[0, 2, 3]).unwrap()
    );

    match mut_elems(&mut test_array, &[4]) {
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

    match mut_elems(&mut test_array, &[1, 2, 1]) {
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

    let es = mut_elems(&mut test_array, &[1, 3]).unwrap();
    *es[0] = 5;
    *es[1] = 7;
    assert_eq!([1, 5, 3, 7], test_array);
}

pub fn as_mut_elems<const N: usize, T>(target: &mut [T; N]) -> [&mut T; N] {
    // Safety: `from_fn()` guarantees that indices `i` 
    // are in-bounds and unique.
    std::array::from_fn(|i| unsafe {
        &mut *(target.get_unchecked_mut(i) as *mut T)
    })
}

#[test]
fn test_as_mut_elems() {
    let mut test_array = [1u8, 2, 3, 4];
    let es = as_mut_elems(&mut test_array);
    assert_eq!([&1, &2, &3, &4], es);
    *es[1] = 5;
    *es[3] = 7;
    assert_eq!([1, 5, 3, 7], test_array);
}
