use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use crate::binary::{from_reader, to_writer, Deserializer, Serializer};
use crate::{byte_array, int_array, long_array, Compound, List, Value};

const ROOT_NAME: &str = "The root name‽";

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Struct {
    byte: i8,
    list_of_int: Vec<i32>,
    list_of_string: Vec<String>,
    string: String,
    inner: Inner,
    #[serde(with = "int_array")]
    int_array: Vec<i32>,
    #[serde(with = "byte_array")]
    byte_array: Vec<i8>,
    #[serde(with = "long_array")]
    long_array: Vec<i64>,
    some_int: Option<i32>,
    none_int: Option<i32>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Inner {
    int: i32,
    long: i64,
    float: f32,
    double: f64,
}

impl Struct {
    pub fn new() -> Self {
        Self {
            byte: 123,
            list_of_int: vec![3, -7, 5],
            list_of_string: vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
            string: "aé日".to_owned(),
            inner: Inner {
                int: i32::MIN,
                long: i64::MAX,
                float: 1e10_f32,
                double: f64::NEG_INFINITY,
            },
            int_array: vec![5, -9, i32::MIN, 0, i32::MAX],
            byte_array: vec![0, 1, 2],
            long_array: vec![123, 456, 789],
            some_int: Some(321),
            none_int: None,
        }
    }

    pub fn value() -> Value {
        Value::Compound(
            Compound::from_iter([
                ("byte".into(), 123_i8.into()),
                ("list_of_int".into(), List::Int(vec![3, -7, 5]).into()),
                (
                    "list_of_string".into(),
                    List::String(vec!["foo".into(), "bar".into(), "baz".into()]).into(),
                ),
                ("string".into(), "aé日".into()),
                (
                    "inner".into(),
                    Compound::from_iter([
                        ("int".into(), i32::MIN.into()),
                        ("long".into(), i64::MAX.into()),
                        ("float".into(), 1e10_f32.into()),
                        ("double".into(), f64::NEG_INFINITY.into()),
                    ])
                    .into(),
                ),
                (
                    "int_array".into(),
                    vec![5, -9, i32::MIN, 0, i32::MAX].into(),
                ),
                ("byte_array".into(), vec![0_i8, 1, 2].into()),
                ("long_array".into(), vec![123_i64, 456, 789].into()),
                ("some_int".into(), 321.into()),
            ])
            .into(),
        )
    }
}

#[test]
fn round_trip_binary_struct() {
    let mut buf = Vec::new();

    let struct_ = Struct::new();

    struct_
        .serialize(&mut Serializer::new(&mut buf, ROOT_NAME))
        .unwrap();

    let reader = &mut buf.as_slice();

    let mut de = Deserializer::new(reader, true);

    let struct_de = Struct::deserialize(&mut de).unwrap();

    assert_eq!(struct_, struct_de);
    assert_eq!(de.root_name, ROOT_NAME);
}

#[test]
fn round_trip_binary_value() {
    let mut buf = Vec::new();

    let value = Struct::value();

    value
        .serialize(&mut Serializer::new(&mut buf, ROOT_NAME))
        .unwrap();

    let reader = &mut buf.as_slice();

    let mut de = Deserializer::new(reader, true);

    let value_de = Value::deserialize(&mut de).unwrap();

    assert_eq!(value, value_de);
    assert_eq!(de.root_name, ROOT_NAME);
}

#[test]
fn to_hematite() {
    let mut buf = Vec::new();

    let struct_ = Struct::new();

    struct_
        .serialize(&mut Serializer::new(&mut buf, ROOT_NAME))
        .unwrap();

    let struct_de: Struct = nbt::from_reader(&mut buf.as_slice()).unwrap();

    assert_eq!(struct_, struct_de);
}

#[test]
fn root_requires_compound() {
    let mut buf = Vec::new();
    assert!(123
        .serialize(&mut Serializer::new(&mut buf, ROOT_NAME))
        .is_err());
}

#[test]
fn mismatched_array_element() {
    #[derive(Serialize)]
    struct Struct {
        #[serde(with = "byte_array")]
        data: Vec<i32>,
    }

    let struct_ = Struct {
        data: vec![1, 2, 3],
    };

    let mut buf = Vec::new();
    assert!(struct_
        .serialize(&mut Serializer::new(&mut buf, ROOT_NAME))
        .is_err());
}

#[test]
fn struct_to_value() {
    let mut buf = Vec::new();

    let struct_ = Struct::new();

    to_writer(&mut buf, &struct_).unwrap();

    let val: Value = from_reader(&mut buf.as_slice()).unwrap();

    assert_eq!(val, Struct::value());
}

#[test]
fn value_to_struct() {
    let mut buf = Vec::new();

    to_writer(&mut buf, &Struct::value()).unwrap();

    let struct_: Struct = from_reader(&mut buf.as_slice()).unwrap();

    assert_eq!(struct_, Struct::new());
}

#[test]
fn value_from_json() {
    let mut struct_ = Struct::new();

    // JSON numbers only allow finite floats.
    struct_.inner.double = 12345.0;

    let string = serde_json::to_string_pretty(&struct_).unwrap();

    let struct_de: Struct = serde_json::from_str(&string).unwrap();

    assert_eq!(struct_, struct_de);
}

/// Serves as documented examples of NBT files.
mod binary {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn simple_struct() {
        #[derive(Serialize)]
        struct A {
            a: i8,
            bc: i16,
        }

        let s = A { a: 10, bc: 258 };

        let mut buf = Vec::new();

        to_writer(&mut buf, &s).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf[..],
            [
                10_u8,  // Compound
                0, 0,   // Compound's name has length of 0 (unnamed)
                // First field
                1,      // Signed byte
                0, 1,   // The name is 1 byte long
                b'a',   // The name is 'a'
                10,     // Value is 10
                // Second field
                2,      // Signed short
                0, 2,   // The name is 2 bytes long 
                b'b', b'c',     // The name is 'bc'
                1, 2,   // Value is 258 (big endian) 
                0,      // End of compound (root)
            ]
        );
    }

    #[test]
    fn simple_array() {
        #[derive(Serialize)]
        struct A {
            #[serde(with = "int_array")]
            x: Vec<i32>,
        }

        let s = A { x: vec![1, 2, 3] };

        let mut buf = Vec::new();

        to_writer(&mut buf, &s).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf[..],
            [
                10_u8,  // Compound
                0, 0,   // Compound's name has length of 0 (unnamed)
                // First field
                11,         // Int array
                0, 1,       // The name is 1 byte long
                b'x',       // The name is 'x'
                0, 0, 0, 3, // Array contains 3 items
                0, 0, 0, 1, // First item
                0, 0, 0, 2, // Second item
                0, 0, 0, 3, // Third item
                0,      // End of compound (root)
            ]
        );
    }

    #[test]
    fn nested_compounds() {
        #[derive(Serialize)]
        struct Outer {
            field: Inner,
        }

        #[derive(Serialize)]
        struct Inner {
            byte: i8,
        }

        let s = Outer {
            field: Inner { byte: 8 },
        };

        let mut buf = Vec::new();

        to_writer(&mut buf, &s).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            buf[..],
            [
                10_u8,  // Compound
                0, 0,   // Compound's name has length of 0 (unnamed)
                // First field
                10,         // Compound
                0, 5,       // Name is 'field'
                b'f', b'i', b'e', b'l', b'd',
                // Inner first field
                1,
                0, 4,
                b'b', b'y', b't', b'e',
                8,
                0,      // End of compound (inner)
                0,      // End of compound (root)
            ]
        );
    }
}
