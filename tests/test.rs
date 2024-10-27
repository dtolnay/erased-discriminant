use erased_discriminant::Discriminant;
use std::collections::HashSet;

enum Enum<'a> {
    A(#[allow(dead_code)] &'a str),
    B,
}

enum DifferentEnum {
    A,
}

#[test]
fn test_eq() {
    let temporary_string = "...".to_owned();
    let a = Enum::A(&temporary_string);
    let b = Enum::B;
    let a_discriminant = Discriminant::of(&a);
    let b_discriminant = Discriminant::of(&b);
    drop(temporary_string);
    assert_eq!(a_discriminant, a_discriminant);
    assert_ne!(a_discriminant, b_discriminant);

    let different_discriminant = Discriminant::of(&DifferentEnum::A);
    assert_ne!(a_discriminant, different_discriminant);
    assert_ne!(b_discriminant, different_discriminant);
}

#[test]
fn test_hashset() {
    let mut set = HashSet::new();

    let temporary_string = "...".to_owned();
    set.insert(Discriminant::of(&Enum::A(&temporary_string)));
    set.insert(Discriminant::of(&Enum::B));
    set.insert(Discriminant::of(&DifferentEnum::A));
    drop(temporary_string);
    assert_eq!(set.len(), 3);

    set.insert(Discriminant::of(&Enum::A("other string")));
    set.insert(Discriminant::of(&Enum::B));
    set.insert(Discriminant::of(&DifferentEnum::A));
    assert_eq!(set.len(), 3);

    assert_eq!(set, set.clone());
}
