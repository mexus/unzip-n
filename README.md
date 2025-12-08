<!-- cargo-rdme start -->

[![CI](https://github.com/mexus/unzip-n/actions/workflows/ci.yml/badge.svg)](https://github.com/mexus/unzip-n/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/unzip-n.svg)](https://crates.io/crates/unzip-n)
[![docs.rs](https://docs.rs/unzip-n/badge.svg)](https://docs.rs/unzip-n)

Procedural macro for unzipping iterators-over-`n`-length-tuples into `n` collections.

Here's a brief example of what it is capable of:

```rust
use unzip_n::unzip_n;

unzip_n!(pub 3);
use Unzip3 as _; // Just a verification of the trait name.
// Or simply leave the visibility modifier absent for inherited visibility
// (which usually means "private").
// unzip_n!(3);

let v = vec![(1, 2, 3), (4, 5, 6)];
let (v1, v2, v3) = v.into_iter().unzip_n_vec();

assert_eq!(v1, &[1, 4]);
assert_eq!(v2, &[2, 5]);
assert_eq!(v3, &[3, 6]);
```

Or you could give the trait an explicit name:

```rust
use unzip_n::unzip_n;

unzip_n!(pub MyTrait 3);
use MyTrait as _; // Just a verification that the trait is generated with the desired name.

let v = vec![(1, 2, 3), (4, 5, 6)];
let (v1, v2, v3) = v.into_iter().unzip_n_vec();

assert_eq!(v1, &[1, 4]);
assert_eq!(v2, &[2, 5]);
assert_eq!(v3, &[3, 6]);
```

Or generate multiple traits at once using a range:

```rust
use unzip_n::unzip_n;

unzip_n!(pub 2..=4);  // Generates Unzip2, Unzip3, Unzip4

let pairs = vec![(1, 2), (3, 4)];
let (v1, v2) = pairs.into_iter().unzip_n_vec();
assert_eq!(v1, &[1, 3]);
assert_eq!(v2, &[2, 4]);

let triples = vec![(1, 2, 3), (4, 5, 6)];
let (v1, v2, v3) = triples.into_iter().unzip_n_vec();
assert_eq!(v1, &[1, 4]);
assert_eq!(v2, &[2, 5]);
assert_eq!(v3, &[3, 6]);
```

With an explicit trait name prefix:

```rust
use unzip_n::unzip_n;

unzip_n!(pub MyUnzip 2..4);  // Generates MyUnzip2, MyUnzip3
use MyUnzip2 as _;
use MyUnzip3 as _;
```

# License

Licensed under either of

* Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

<!-- cargo-rdme end -->
