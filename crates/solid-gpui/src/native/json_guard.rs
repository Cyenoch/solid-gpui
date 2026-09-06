use serde::ser::{self, Serialize};

pub struct Guard;
type Error = serde_json::Error;
fn invalid(message: &str) -> Error {
    ser::Error::custom(message)
}

macro_rules! scalar {
    ($($name:ident($ty:ty)),* $(,)?) => { $(fn $name(self, _: $ty) -> Result<(), Error> { Ok(()) })* };
}
impl ser::Serializer for Guard {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_char(char),
        serialize_str(&str),
        serialize_bytes(&[u8])
    );
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.serialize_i128(v.into())
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.serialize_u128(v.into())
    }
    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        if !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&v) {
            Err(invalid("integer exceeds JavaScript's exact JSON range"))
        } else {
            Ok(())
        }
    }
    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        if v > 9_007_199_254_740_991 {
            Err(invalid("integer exceeds JavaScript's exact JSON range"))
        } else {
            Ok(())
        }
    }
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.serialize_f64(v.into())
    }
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        if v.is_finite() && (v.fract() != 0.0 || v.abs() <= 9_007_199_254_740_991.0) {
            Ok(())
        } else {
            Err(invalid(
                "non-finite or unsafe integer numbers are not native JSON values",
            ))
        }
    }
    fn serialize_none(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Error> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Error> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<(), Error> {
        v.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_tuple(self, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self, Error> {
        Ok(self)
    }
}
macro_rules! compound {
    ($trait:ident, $method:ident $(, $key:ident: $key_ty:ty)?) => {
        impl ser::$trait for Guard {
            type Ok = ();
            type Error = Error;
            fn $method<T: Serialize + ?Sized>(&mut self, $($key: $key_ty,)? value: &T) -> Result<(), Error> { $(let _ = $key;)? value.serialize(Guard) }
            fn end(self) -> Result<(), Error> { Ok(()) }
        }
    };
}
compound!(SerializeSeq, serialize_element);
compound!(SerializeTuple, serialize_element);
compound!(SerializeTupleStruct, serialize_field);
compound!(SerializeTupleVariant, serialize_field);
compound!(SerializeStruct, serialize_field, key: &'static str);
compound!(SerializeStructVariant, serialize_field, key: &'static str);
impl ser::SerializeMap for Guard {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        key.serialize(Guard)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(Guard)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}
