# Type-erased `Discriminant`

This crate provides a `Discriminant` type that behaves like
`core::mem::Discriminant<T>` but without the generic type parameter `T`. With
this, we can build collections such as HashSet that contain discriminants from a
mixture of different enum types.

```rust
use erased_discriminant::Discriminant;
use std::collections::HashSet;

enum Enum {
    A(i32),
    B,
}

enum DifferentEnum {
    A,
}

let mut set = HashSet::new();
set.insert(Discriminant::of(&Enum::A(99)));
set.insert(Discriminant::of(&Enum::B));
set.insert(Discriminant::of(&DifferentEnum::A));
assert_eq!(set.len(), 3);
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
