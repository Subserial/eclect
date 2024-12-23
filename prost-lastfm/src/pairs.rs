use serde::Serialize;
use serde::ser::Impossible;

pub struct InvalidStructError(String);

impl std::fmt::Debug for InvalidStructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for InvalidStructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for InvalidStructError {}
impl serde::ser::Error for InvalidStructError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        InvalidStructError(msg.to_string())
    }
}

pub fn to_pairs<T: Serialize>(val: T) -> Result<Vec<(String, String)>, InvalidStructError> {
    val.serialize(KeyValueSerializer)
}

macro_rules! string_serialize {
    ($fn:ident, $ty:ty) => {
        fn $fn(self, v: $ty) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string())
        }
    };
}

struct FlatSerializer;

impl<'s> serde::Serializer for FlatSerializer {
    type Ok = String;
    type Error = InvalidStructError;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok((if v { "0" } else { "1" }).to_string())
    }

    string_serialize!(serialize_i8, i8);
    string_serialize!(serialize_i16, i16);
    string_serialize!(serialize_i32, i32);
    string_serialize!(serialize_i64, i64);
    string_serialize!(serialize_u8, u8);
    string_serialize!(serialize_u16, u16);
    string_serialize!(serialize_u32, u32);
    string_serialize!(serialize_u64, u64);
    string_serialize!(serialize_f32, f32);
    string_serialize!(serialize_f64, f64);
    string_serialize!(serialize_char, char);
    string_serialize!(serialize_str, &str);

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(String::from_utf8_lossy(v).to_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok("".to_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(InvalidStructError(
            "cannot serialize externally tagged variant".to_string(),
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(InvalidStructError("cannot serialize sequence".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(InvalidStructError("cannot serialize tuple".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(InvalidStructError(
            "cannot serialize tuple struct".to_string(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(InvalidStructError("cannot serialize sequence".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(InvalidStructError("cannot serialize map".to_string()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(InvalidStructError("cannot serialize struct".to_string()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(InvalidStructError("cannot serialize struct".to_string()))
    }
}

macro_rules! err_serialize {
    ($fn:ident) => {
        fn $fn(self) -> Result<Self::Ok, Self::Error> {
            Err(InvalidStructError(format!("cannot serialize pairs from None or unit")))
        }
    };
    ($fn:ident, $ty:ty) => {
        fn $fn(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
            Err(InvalidStructError(format!(
                "cannot serialize pairs from {}",
                stringify!($ty)
            )))
        }
    };
    ($fn:ident, $complex:expr, $($args:expr),+) => {
        fn $fn(self, $($args),+) -> Result<Self::Ok, Self::Error> {
            Err(InvalidStructError(format!(
                "cannot serialize pairs from {}",
                stringify!($complex)
            )))
        }
    };
}

struct KeyValueSerializer;

impl serde::Serializer for KeyValueSerializer {
    type Ok = Vec<(String, String)>;
    type Error = InvalidStructError;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = KeyValueDataSerializer;
    type SerializeStruct = KeyValueDataSerializer;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    err_serialize!(serialize_bool, bool);
    err_serialize!(serialize_i8, i8);
    err_serialize!(serialize_i16, i16);
    err_serialize!(serialize_i32, i32);
    err_serialize!(serialize_i64, i64);
    err_serialize!(serialize_u8, u8);
    err_serialize!(serialize_u16, u16);
    err_serialize!(serialize_u32, u32);
    err_serialize!(serialize_u64, u64);
    err_serialize!(serialize_f32, f32);
    err_serialize!(serialize_f64, f64);
    err_serialize!(serialize_char, char);
    err_serialize!(serialize_str, &str);
    err_serialize!(serialize_bytes, &[u8]);
    err_serialize!(serialize_none);
    err_serialize!(serialize_unit);
    err_serialize!(serialize_unit_struct, &'static str);

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(InvalidStructError(
            "cannot serialize pairs from unit variant".to_string(),
        ))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(vec![(name.to_string(), value.serialize(FlatSerializer)?)])
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = variant.serialize(FlatSerializer)?;
        let value = value.serialize(FlatSerializer)?;
        Ok(vec![(key, value)])
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(InvalidStructError(
            "cannot construct pairs from sequence".to_string(),
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(InvalidStructError(
            "cannot construct pairs from tuple".to_string(),
        ))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(InvalidStructError(
            "cannot construct pairs from tuple".to_string(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(InvalidStructError(
            "cannot construct pairs from tuple".to_string(),
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(KeyValueDataSerializer::default())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(KeyValueDataSerializer::default())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(InvalidStructError(
            "cannot construct pairs from externally tagged struct".to_string(),
        ))
    }
}

#[derive(Default)]
struct KeyValueDataSerializer {
    pairs: Vec<(String, String)>,
}

impl<'s> serde::ser::SerializeMap for KeyValueDataSerializer {
    type Ok = Vec<(String, String)>;
    type Error = InvalidStructError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(InvalidStructError("cannot serialize key alone".to_string()))
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(InvalidStructError(
            "cannot serialize value alone".to_string(),
        ))
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.pairs.push((
            key.serialize(FlatSerializer)?,
            value.serialize(FlatSerializer)?,
        ));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.pairs)
    }
}

impl<'s> serde::ser::SerializeStruct for KeyValueDataSerializer {
    type Ok = Vec<(String, String)>;
    type Error = InvalidStructError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(FlatSerializer)?;
        if value != "" {
            self.pairs.push((key.to_string(), value));
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.pairs)
    }
}
