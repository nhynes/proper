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
