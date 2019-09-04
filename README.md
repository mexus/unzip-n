# unzip-n

Procedural macro for unzipping iterators-over-`n`-length-tuples into `n` collections.

Here's a brief example of what it is capable of:

```rust
use unzip_n::unzip_n;

unzip_n!(3);

fn main() {
    let v = vec![(1, 2, 3), (4, 5, 6)];
    let (v1, v2, v3) = v.into_iter().unzip_n_vec();

    assert_eq!(v1, &[1, 4]);
    assert_eq!(v2, &[2, 5]);
    assert_eq!(v3, &[3, 6]);
}
```
