#![allow(dead_code)]

#[macro_use] extern crate yade;

use std::error::Error;

#[derive(Debug, YadeError)]
pub struct UnitStruct;

#[derive(Debug, YadeError)]
pub struct TupleStruct(i32);

#[derive(Debug, YadeError)]
#[display(msg = "a kind error: {}", kind)]
pub struct KindError {
    pub kind: KindErrorKind,

    #[cause]
    pub cause: Option<Box<Error>>,
}

#[derive(Debug, PartialEq, YadeKind)]
pub enum KindErrorKind {
    #[display(msg = "Kind is One")]
    One,

    #[display(msg = "Kind is Two: {}", _0)]
    Two(String),
    Three
}

#[derive(Debug, YadeError)]
pub struct BoxedError {
    #[cause]
    pub cause: Box<Error>,
}


#[derive(Debug, YadeError)]
enum EnumError {
    #[display(msg = "Some things here")]
    Hello,

    #[display(msg = "cat: {}", cat)]
    More {
        cat: String,
        #[cause] cause: std::io::Error,
    },

    Last(#[cause] std::io::Error),
}

#[test]
fn boxed_error() {
    let e = BoxedError {
        cause: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "boxed cause")),
    };

    assert_eq!("BoxedError", e.to_string());
    assert_eq!("boxed cause", e.cause().unwrap().to_string());
}

#[test]
fn kind_error() {
    let e = KindError {
        kind: KindErrorKind::Two("Things for free".into()),
        cause: Some(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "for a cause"))),
    };

    assert_eq!("a kind error: Kind is Two: Things for free", e.to_string());

    assert_eq!("for a cause", e.cause().unwrap().to_string());
}

#[test]
fn kind_error_nocause() {
    let e = KindError {
        kind: KindErrorKind::One,
        cause: None,
    };

    assert_eq!("a kind error: Kind is One", e.to_string());

    assert!(e.cause().is_none());
}

#[test]
fn enum_error() {
    let e = EnumError::More {
        cat: "is not a dog".into(),
        cause: std::io::Error::new(std::io::ErrorKind::Other, "oh no!"),
    };

    assert_eq!("cat: is not a dog", e.to_string());

    assert_eq!("oh no!", e.cause().unwrap().to_string());
}


#[test]
fn tuple_struct_enum() {
    let res = std::fs::File::open("a_missing_file.txt")
        .map_err(|e| EnumError::Last(e));

    match res {
        Err(err) => {
            assert_eq!("Last", err.to_string());
            assert_eq!("No such file or directory (os error 2)", err.cause().unwrap().to_string());

        },
        _ => panic!("Expected to find an error"),
    };
}
