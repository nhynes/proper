#![cfg(test)]

#[macro_use]
extern crate proper;

#[derive(Prim, Clone)]
struct TestStruct(u32);

use std::convert::TryFrom;

#[derive(Prim, PartialEq, Debug)]
enum TestEnum {
    Variant0,
    Variant1,
}

#[derive(Prim, PartialEq, Debug)]
#[prim(ty = "u16")]
enum TestEnumU16 {
    Variant0,
    Variant1,
}

#[test]
fn test_struct() {
    let s = TestStruct::from(1u8);
    assert_eq!(<u32>::from(s.clone()), 1u32);
    assert_eq!(<u64>::from(s.clone()), 1u64);

    let s = TestStruct::from(1u16);
    assert_eq!(<u32>::from(s.clone()), 1u32);
    assert_eq!(<u64>::from(s.clone()), 1u64);

    let s = TestStruct::from(1u32);
    assert_eq!(<u32>::from(s.clone()), 1u32);
    assert_eq!(<u64>::from(s.clone()), 1u64);
}

#[test]
fn test_enum() {
    let e = TestEnum::try_from(1u8);
    assert_eq!(e, Ok(TestEnum::Variant1));
}

#[derive(Prim, PartialEq, Debug)]
#[prim(ty = "i32")]
enum TestExplicit {
    Foo = 123,
    Bar = 456,
    Baz,
    Backwards = 42,
    AfterBackwards,
}

#[test]
fn test_explicit() {
    assert_eq!(123, TestExplicit::Foo as u32);
    assert_eq!(456, TestExplicit::Bar as u32);
    assert_eq!(457, TestExplicit::Baz as u32);
    assert_eq!(42, TestExplicit::Backwards as u32);
    assert_eq!(43, TestExplicit::AfterBackwards as u32);

    assert!(TestExplicit::try_from(0).is_err());
    assert_eq!(TestExplicit::try_from(42), Ok(TestExplicit::Backwards));
    assert_eq!(TestExplicit::try_from(43), Ok(TestExplicit::AfterBackwards));
    assert!(TestExplicit::try_from(44).is_err());

    assert_eq!(TestExplicit::try_from(123), Ok(TestExplicit::Foo));
    assert!(TestExplicit::try_from(124).is_err());
    assert_eq!(TestExplicit::try_from(456), Ok(TestExplicit::Bar));
    assert_eq!(TestExplicit::try_from(457), Ok(TestExplicit::Baz));
    assert!(TestExplicit::try_from(458).is_err());
}
