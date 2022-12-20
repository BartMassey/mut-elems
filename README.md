![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
[![CI](https://github.com/BartMassey/mut-elems/actions/workflows/main.yml/badge.svg)](https://github.com/BartMassey/mut-elems/actions)
[![crates-io](https://img.shields.io/crates/v/mut-elems.svg)](https://crates.io/crates/mut-elems)
[![api-docs](https://docs.rs/mut-elems/badge.svg)](https://docs.rs/mut-elems)

# mut-elems: get simultaneous mutable access to multiple elements of a mutable array, slice or `Vec`
Bart Massey 2022 (version 0.1.0)


Get simultaneous mutable access to multiple elements of a
mutable array, slice or `Vec`. This is a generalization of
[slice::split_at_mut] to individual elements rather than
just a pair of
subslices. [API docs](https://bartmassey.github.io/mut-elems/)
are available.

## Examples

```rust
use mut_elems::*;

let mut a = [1u8, 2, 3, 4];
    let es = a.mut_elems(&[1, 3]).unwrap();
    *es[0] = 5;
    *es[1] = 7;
    assert_eq!([1, 5, 3, 7], a);
```


# License

This crate is made available under the "MIT license". Please
see the file `LICENSE.txt` in this distribution for license
terms.

# Acknowledgments

Thanks to the `cargo-readme` crate for generation of this `README`.
