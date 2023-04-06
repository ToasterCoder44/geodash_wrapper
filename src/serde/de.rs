use std::{
    path::Path,
    fs::File,
    io::{Read, BufReader},
    sync::Arc
};
use serde::{
    de::{self, MapAccess, SeqAccess},
    serde_if_integer128
};

use xorstream::Transformer as XorReader;
use base64::{
    read::DecoderReader as Base64Reader,
    engine::{GeneralPurpose, general_purpose::URL_SAFE}
};
use libflate::gzip::Decoder as GzipReader;
use quick_xml::{
    Reader as XmlReader,
    Result as XmlResult,
    events::Event as XmlEvent
};

use super::error::{ DeError, DeResult };

type DecodedDataReader<'de, R> =
    GzipReader<
        Base64Reader<
            'de,
            GeneralPurpose,
            XorReader<R>
        >
    >;

type DecodedDataXmlReader<'de, R> =
XmlReader<
    BufReader<
        DecodedDataReader<'de, R>
    >
>;

#[derive(Debug)]
pub struct Header { // move to serde
    pub xml_version: String,
    pub plist_version: String,
    pub gj_version: String
}

#[derive(Debug)]
pub struct DataWithHeader<T> {
    pub t: T,
    pub header: Header
}

pub struct Deserializer<'de, R: Read> {
    reader: DecodedDataXmlReader<'de, R>,
    buffer: Vec<u8>,
    header: Header,
    peeked_next: Option<Arc<Event>>,
    is_instant_dict_end: bool,
    is_eof: bool
}

impl<'de, R: Read> Deserializer<'de, R> {
    fn decode(reader: R) -> DeResult<DecodedDataReader<'de, R>> {
        let reader = XorReader::new(vec![11], reader);
        let reader = Base64Reader::new(reader, &URL_SAFE);
        if let Ok(reader) = GzipReader::new(reader) { Ok(reader) }
        else { return Err(DeError::Read); }
    }

    pub fn from_reader(reader: R) -> DeResult<Self> {
        let reader = Self::decode(reader)?;
        let reader = XmlReader::from_reader(BufReader::new(reader));
        Ok(Self {
            reader,
            buffer: vec![],
            header: Header {
                xml_version: String::new(),
                plist_version: String::new(),
                gj_version: String::new()
            },
            peeked_next: None,
            is_instant_dict_end: false,
            is_eof: false
        })
    }
}

impl<'de> Deserializer<'de, File> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> DeResult<Self> {
        if let Ok(file) = File::open(path) {
            Self::from_reader(file)
        } else { Err(DeError::Open) }
    }
}

pub fn from_reader<'de, T, R: Read>(reader: R) -> DeResult<DataWithHeader<T>>
where T: de::Deserialize<'de> {
    let mut deserializer = Deserializer::from_reader(reader)?;
    deserializer.skip_header()?;
    let result = T::deserialize(&mut deserializer)?;
    if let Ok(event) = deserializer.next() {
        if let Event::Eof = *event {
            Ok(DataWithHeader {
                t: result,
                header: deserializer.header
            })
        } else { Err(DeError::ExpectedEof) }
    }
    else { Err(DeError::ExpectedEof) }
}

pub fn from_file<'de, T, P: AsRef<Path>>(path: P) -> DeResult<DataWithHeader<T>>
where T: de::Deserialize<'de> {
    let mut deserializer = Deserializer::from_file(path)?;
    deserializer.skip_header()?;
    let result = T::deserialize(&mut deserializer)?;
    if let Ok(event) = deserializer.next() {
        if let Event::Eof = *event {
            Ok(DataWithHeader {
                t: result,
                header: deserializer.header
            })
        } else { Err(DeError::ExpectedEof) }
    }
    else { Err(DeError::ExpectedEof) }
}

#[derive(PartialEq, Debug)]
enum Event {
    XmlVersion(String),
    PlistStart {
        plist_version: String,
        gj_version: String
    },
    DictStart,
    DictEnd,
    Key(String),
    String(String),
    Integer(String),
    Real(String),
    True,
    Eof
}

enum PreEvent {
    None,
    Key,
    String,
    Integer,
    Real
}

macro_rules! save_next_peek {
    ($self: expr, $event: expr) => {
        {
            $self.peeked_next = Some(Arc::new($event));
            return Ok::<(), DeError>(())
        }
    };
}

impl<'a, 'de, R: Read> Deserializer<'de, R> {
    fn xml_next(&'a mut self) -> XmlResult<XmlEvent<'a>> {
        self.reader.read_event_into(&mut self.buffer)
    }

    fn save_next_peek(&'a mut self) -> DeResult<()> {
        if self.is_instant_dict_end {
            self.is_instant_dict_end = false;
            save_next_peek!(self, Event::DictEnd);
        }
        // deserializer always throws an error if an Eof event occured
        if self.is_eof { panic!("Tried to read event after receiving EOF") }
        let mut expect = PreEvent::None;

        loop {
            match self.xml_next() {
                Ok(event) => {
                    match event {
                        XmlEvent::Decl(decl) => {
                            if let Ok(version) = decl.version() {
                                if let Ok(version) = String::from_utf8(version.to_vec()) {
                                    save_next_peek!(self, Event::XmlVersion(version))
                                }
                            }
                            return Err(DeError::NoXmlVersionInfo);
                        }
                        XmlEvent::Start(tag) => {
                            if let PreEvent::None = expect {
                                match tag.name().into_inner() {
                                    b"plist" => {
                                        let mut plist_version: Option<String> = None;
                                        let mut gj_version: Option<String> = None;

                                        for attr in tag.attributes() {
                                            match attr {
                                                Ok(attr) => {
                                                    match attr.key.into_inner() {
                                                        b"version" => {
                                                            match attr.unescape_value() {
                                                                Ok(attr) => {
                                                                    plist_version = Some(attr.to_string());
                                                                }
                                                                Err(err) => {
                                                                    return Err(DeError::XmlParse(err))
                                                                }
                                                            }
                                                        }
                                                        b"gjver" => {
                                                            match attr.unescape_value() {
                                                                Ok(attr) => {
                                                                    gj_version = Some(attr.to_string());
                                                                }
                                                                Err(err) => {
                                                                    return Err(DeError::XmlParse(err))
                                                                }
                                                            }
                                                        }
                                                        _ => { return Err(DeError::UnexpectedXmlAttr) }
                                                    }
                                                }
                                                Err(err) => { return Err(DeError::XmlAttrParse(err)) }
                                            }
                                        }

                                        let plist_version = match plist_version {
                                            Some(version) => version,
                                            None => { return Err(DeError::ExpectedPlistVersion) },
                                        };
                                        let gj_version = match gj_version {
                                            Some(version) => version,
                                            None => { return Err(DeError::ExpectedPlistVersion) },
                                        };

                                        save_next_peek!(self, Event::PlistStart {
                                            plist_version,
                                            gj_version
                                        })
                                    }
                                    b"d" | b"dict" => { save_next_peek!(self, Event::DictStart) }
                                    b"k" => { expect = PreEvent::Key }
                                    b"s" => { expect = PreEvent::String }
                                    b"i" => { expect = PreEvent::Integer }
                                    b"r" => { expect = PreEvent::Real }
                                    _ => { return Err(DeError::UnknownXmlTag) }
                                }
                            } else { return Err(DeError::UnexpectedXmlTag) }
                        }
                        XmlEvent::End(tag) => {
                            if let PreEvent::None = expect {
                                match tag.name().into_inner() {
                                    b"plist" | b"k" | b"s" | b"i" | b"r" => {}
                                    b"d" | b"dict" => { save_next_peek!(self, Event::DictEnd) }
                                    _ => { return Err(DeError::UnknownXmlTag) }
                                }
                            } else { return Err(DeError::UnexpectedXmlTag) }
                        }
                        XmlEvent::Empty(tag) => {
                            if let PreEvent::None = expect {
                                match tag.name().into_inner() {
                                    b"d" | b"dict" => {
                                        self.is_instant_dict_end = true;
                                        save_next_peek!(self, Event::DictStart);
                                    }
                                    b"t" => { save_next_peek!(self, Event::True) }
                                    _ => { return Err(DeError::UnknownXmlTag) }
                                }
                            } else { return Err(DeError::UnexpectedXmlTag); }
                        }
                        XmlEvent::Text(text) => {
                            if let PreEvent::None = expect { return Err(DeError::UnexpectedXmlText) }
                            else {
                                match text.unescape() {
                                    Ok(text) => match expect {
                                        PreEvent::None => { unreachable!() }
                                        PreEvent::Key => {
                                            save_next_peek!(self, Event::Key(text.to_string()))
                                        }
                                        PreEvent::String => {
                                            save_next_peek!(self, Event::String(text.to_string()))
                                        }
                                        PreEvent::Integer => {
                                            save_next_peek!(self, Event::Integer(text.to_string()))
                                        }
                                        PreEvent::Real => {
                                            save_next_peek!(self, Event::Real(text.to_string()))
                                        }
                                    }
                                    Err(err) => { return Err(DeError::XmlParse(err)) }
                                }
                            }
                        }
                        XmlEvent::Eof => {
                            self.is_eof = true;
                            save_next_peek!(self, Event::Eof)
                        }
                        _ => { return Err(DeError::UnexpectedOtherXml) }
                    }
                }
                Err(error) => { return Err(DeError::XmlParse(error)) }
            }
        }
    }

    fn peek(&'a mut self) -> DeResult<&Event> {
        if let None = self.peeked_next {
            self.save_next_peek()?;
        }
        if let Some(peeked) = &self.peeked_next {
            Ok(peeked)
        } else { unreachable!() }
    }
    
    fn next(&'a mut self) -> DeResult<Arc<Event>> {
        if let None = self.peeked_next {
            self.save_next_peek()?;
        }
        if let Some(peeked) = &self.peeked_next {
            let peeked = Arc::clone(&peeked);
            self.peeked_next = None;
            Ok(peeked)
        } else { unreachable!() }
    }
}

macro_rules! deserialize_type {
    ($deserialize: ident => $visit: ident, $true: expr) => {
        fn $deserialize<V>(self, visitor: V) -> DeResult<V::Value>
        where V: de::Visitor<'de> {
            match &*self.next()? {
                Event::String(text) |
                Event::Key(text) |
                Event::Integer(text) |
                Event::Real(text) => {
                    if let Ok(parsed) = text.parse() { visitor.$visit(parsed) }
                    else { Err(DeError::Deserialization) }
                }
                Event::True => { visitor.$visit($true) }
                _ => Err(DeError::Deserialization)
            }
        }
    };
}

struct DictReader<'a, 'de, R: Read> {
    de: &'a mut Deserializer<'de, R>
}

impl<'a, 'de, R: Read> DictReader<'a, 'de, R> {
    fn new(de: &'a mut Deserializer<'de, R>) -> Self {
        Self { de }
    }
}

impl<'a, 'de, R: Read> MapAccess<'de> for DictReader<'a, 'de, R> {
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> DeResult<Option<K::Value>>
    where K: de::DeserializeSeed<'de> {
        match self.de.peek()? {
            Event::DictEnd => Ok(None),
            Event::Key(_) => Ok(Some(seed.deserialize(&mut *self.de)?)),
            _ => Err(DeError::Deserialization)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> DeResult<V::Value>
    where V: de::DeserializeSeed<'de> {
        match self.de.peek()? {
            Event::DictStart |
            Event::String(_) |
            Event::Integer(_) |
            Event::Real(_) |
            Event::True => Ok(seed.deserialize(&mut *self.de)?),
            _ => Err(DeError::Deserialization)
        }
    }
}

struct ArrayReader<'a, 'de, R: Read> {
    de: &'a mut Deserializer<'de, R>,
    cur_index: usize
}

impl<'a, 'de, R: Read> ArrayReader<'a, 'de, R> {
    fn new(de: &'a mut Deserializer<'de, R>) -> Self {
        Self { de, cur_index: 0 }
    }
}

impl<'a, 'de, R: Read> SeqAccess<'de> for ArrayReader<'a, 'de, R> {
    type Error = DeError;

    fn next_element_seed<T>(&mut self, seed: T) -> DeResult<Option<T::Value>>
    where T: de::DeserializeSeed<'de> {
        match &*self.de.next()? {
            Event::DictEnd => Ok(None),
            Event::Key(key) => {
                if *key == String::from("k_") + &self.cur_index.to_string() {
                    match self.de.peek()? {
                        Event::DictStart |
                        Event::String(_) |
                        Event::Integer(_) |
                        Event::Real(_) |
                        Event::True => {self.cur_index+=1;Ok(Some(seed.deserialize(&mut *self.de)?))},
                        _ => Err(DeError::Deserialization)
                    }
                } else { Err(DeError::Deserialization) }
            }
            _ => Err(DeError::Deserialization)
        }
    }
}

impl<'de, 'a, R: Read> Deserializer<'de, R> {
    fn skip_header(&mut self) -> DeResult<()> {
        if let Event::XmlVersion(xml_version) = &*self.next()? {
            self.header.xml_version = xml_version.to_string();
            if let Event::PlistStart { plist_version, gj_version } = &*self.next()? {
                self.header.plist_version = plist_version.to_string();
                self.header.gj_version = gj_version.to_string();
            } else {
                panic!(); // idk if it is reachable or not
            }
        } else {
            return Err(DeError::ExpectedXmlVersion);
        }
        Ok(())
    }

    fn deserialize_map_content<V>(&mut self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        let map = visitor.visit_map(DictReader::new(self));
        if let Event::DictEnd = *self.next().unwrap() { map }
        else { unreachable!() }
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<'de, R> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        match self.peek()? {
            Event::DictStart => {
                self.next().unwrap();
                match self.peek()? {
                    Event::Key(key) => {
                        if key == "_isArr" {
                            self.next().unwrap();
                            match *self.next()? {
                                Event::True => visitor.visit_seq(ArrayReader::new(self)),
                                _ => Err(DeError::Deserialization)
                            }

                        } else {
                            self.deserialize_map_content(visitor)
                        }
                    }
                    Event::DictEnd => self.deserialize_map_content(visitor),
                    _ => Err(DeError::Deserialization)
                }
            }
            Event::String(_) => self.deserialize_str(visitor),
            Event::Key(_) => self.deserialize_str(visitor),
            Event::Integer(_) => self.deserialize_i32(visitor),
            Event::Real(_) => self.deserialize_f32(visitor),
            Event::True => self.deserialize_bool(visitor),
            _ => Err(DeError::Deserialization)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        if let Event::True = *self.next()? {
            visitor.visit_bool(true)
        } else {
            Err(DeError::Deserialization)
        }
    }

    deserialize_type!(deserialize_i8 => visit_i8, 1);
    deserialize_type!(deserialize_i16 => visit_i16, 1);
    deserialize_type!(deserialize_i32 => visit_i32, 1);
    deserialize_type!(deserialize_i64 => visit_i64, 1);

    deserialize_type!(deserialize_u8 => visit_u8, 1);
    deserialize_type!(deserialize_u16 => visit_u16, 1);
    deserialize_type!(deserialize_u32 => visit_u32, 1);
    deserialize_type!(deserialize_u64 => visit_u64, 1);

    serde_if_integer128! {
        deserialize_type!(deserialize_i128 => visit_i128, 1);
        deserialize_type!(deserialize_u128 => visit_u128, 1);
    }

    deserialize_type!(deserialize_f32 => visit_f32, 1.0);
    deserialize_type!(deserialize_f64 => visit_f64, 1.0);

    fn deserialize_char<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        match &*self.next()? {
            Event::String(text) |
            Event::Key(text) |
            Event::Integer(text) |
            Event::Real(text) => { visitor.visit_str(&text[..]) }
            Event::True => { visitor.visit_str("true") }
            _ => Err(DeError::Deserialization)
        }
    }

    deserialize_type!(deserialize_string => visit_string, String::from("true"));

    fn deserialize_bytes<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        if let Event::DictStart = *self.next()? {
            if let Event::Key(key) = &*self.next()? {
                if key == "_isArr" {
                    match *self.next()? {
                        Event::True => visitor.visit_seq(ArrayReader::new(self)),
                        _ => Err(DeError::Deserialization)
                    }
                } else { Err(DeError::Deserialization) }
            } else { Err(DeError::Deserialization) }
        } else { Err(DeError::Deserialization) }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        if let Event::DictStart = *self.next()? {
            self.deserialize_map_content(visitor)
        } else { Err(DeError::Deserialization) }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        self.deserialize_any(visitor) // no, its unoptimized
    }
}

#[cfg(test)]
mod tests;
