use std::io::Cursor;
use serde::ser;
use quick_xml::{
    Writer as XMLWriter,
    events::{self as xml_events, Event as XMLEvent}
};
use super::error::{ SerError, SerResult };

pub struct Serializer {
    pub writer: XMLWriter<Cursor<Vec<u8>>> // temporarily public
}

impl Serializer {
    pub fn new() -> Self { // temporarily public
        Self { writer: XMLWriter::new(Cursor::new(vec![])) }
    }
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
        todo!()
    }

    fn serialize_i8(self, v: i8) -> SerResult<()> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> SerResult<()> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> SerResult<()> {
        self.writer.write_event(XMLEvent::Start(xml_events::BytesStart::new("i")));
        self.writer.write_event(XMLEvent::Text(xml_events::BytesText::new(&v.to_string())));
        self.writer.write_event(XMLEvent::End(xml_events::BytesEnd::new("i")));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> SerResult<()> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> SerResult<()> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> SerResult<()> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> SerResult<()> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> SerResult<()> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> SerResult<()> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> SerResult<()> {
        todo!()
    }

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
