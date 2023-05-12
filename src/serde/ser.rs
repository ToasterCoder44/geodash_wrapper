use std::io::Cursor;
use serde::{ser, serde_if_integer128};
use quick_xml::{
    Writer as XmlWriter,
    events::{self as xml_events, Event as XmlEvent}
};
use super::error::{ SerError, SerResult };

pub struct Serializer {
    pub writer: XmlWriter<Cursor<Vec<u8>>> // temporarily public
}

impl Serializer {
    pub fn new() -> Self { // temporarily public
        Self { writer: XmlWriter::new(Cursor::new(vec![])) }
    }
}

macro_rules! write_event {
    ($writer: expr, $event: expr) => {
        if let Err(err) = $writer.write_event($event) {
            return Err(SerError::XmlParse(err))
        }
    };
}

macro_rules! serialize_type {
    ($serialize: ident => $value_type: ident, $tag: expr) => {
        fn $serialize(self, v: $value_type) -> SerResult<()> {
            write_event!(self.writer, XmlEvent::Start(xml_events::BytesStart::new($tag)));
            write_event!(self.writer, XmlEvent::Text(xml_events::BytesText::new(&v.to_string())));
            write_event!(self.writer, XmlEvent::End(xml_events::BytesEnd::new($tag)));
            Ok(())
        }
    };
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> SerResult<()> {
        write_event!(self.writer, XmlEvent::Empty(xml_events::BytesStart::new("i")));
        Ok(())
    }

    serialize_type!(serialize_i8 => i8, "i");
    serialize_type!(serialize_i16 => i16, "i");
    serialize_type!(serialize_i32 => i32, "i");
    serialize_type!(serialize_i64 => i64, "i");

    serialize_type!(serialize_u8 => u8, "i");
    serialize_type!(serialize_u16 => u16, "i");
    serialize_type!(serialize_u32 => u32, "i");
    serialize_type!(serialize_u64 => u64, "i");

    serde_if_integer128! {
        serialize_type!(serialize_u128 => u128, "i");
        serialize_type!(serialize_i128 => i128, "i");
    }

    serialize_type!(serialize_f32 => f32, "r");
    serialize_type!(serialize_f64 => f64, "r");

    fn serialize_char(self, v: char) -> SerResult<()> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> SerResult<()> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> SerResult<()> {
        todo!()
    }

    fn serialize_none(self) -> SerResult<()> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> SerResult<()>
    where
        T: serde::Serialize {
        todo!()
    }

    fn serialize_unit(self) -> SerResult<()> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> SerResult<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> SerResult<()> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> SerResult<()>
    where
        T: serde::Serialize {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> SerResult<()>
    where
        T: serde::Serialize {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> SerResult<Self> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> SerResult<Self> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> SerResult<Self> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerResult<Self> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> SerResult<Self> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> SerResult<Self> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> SerResult<Self> {
        todo!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = SerError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize {
        todo!()
    }

    fn end(self) -> SerResult<()> {
        todo!()
    }
}
