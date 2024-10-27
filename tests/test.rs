use erased_discriminant::Discriminant;

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
