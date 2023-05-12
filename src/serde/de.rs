use std::{
    path::Path,
    fs::File,
    io::{Read, BufReader},
    sync::Arc
};
use serde::{de, serde_if_integer128};

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
    peeked_next: Option<Arc<DeEvent>>,
    is_instant_dict_end: bool,
    is_eof: bool
}

impl<'de, R: Read> Deserializer<'de, R> {
    fn decode(reader: R) -> DeResult<DecodedDataReader<'de, R>> {
        let reader = XorReader::new(vec![11], reader);
        let reader = Base64Reader::new(reader, &URL_SAFE);
        match GzipReader::new(reader) {
            Ok(reader) => Ok(reader),
            Err(err) => Err(DeError::Io(err))
        }
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
        match File::open(path) {
            Ok(file) => Self::from_reader(file),
            Err(err) => Err(DeError::Io(err))
        }
    }
}

pub fn from_reader<'de, T, R: Read>(reader: R) -> DeResult<DataWithHeader<T>>
where T: de::Deserialize<'de> {
    let mut deserializer = Deserializer::from_reader(reader)?;
    deserializer.skip_header()?;
    let result = T::deserialize(&mut deserializer)?;
    if let Ok(event) = deserializer.next() {
        if let DeEvent::Eof = *event {
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
        if let DeEvent::Eof = *event {
            Ok(DataWithHeader {
                t: result,
                header: deserializer.header
            })
        } else { Err(DeError::ExpectedEof) }
    }
    else { Err(DeError::ExpectedEof) }
}

#[derive(PartialEq, Debug)]
enum DeEvent {
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

enum DeEventExpected {
    None,
    Key,
    String,
    Integer,
    Real
}

macro_rules! save_next_peek {
    ($self: expr, $event: expr) => {{
        $self.peeked_next = Some(Arc::new($event));
        return Ok::<(), DeError>(())
    }};
}

impl<'a, 'de, R: Read> Deserializer<'de, R> {
    fn xml_next(&'a mut self) -> XmlResult<XmlEvent<'a>> {
        self.reader.read_event_into(&mut self.buffer)
    }

    fn save_next_peek(&'a mut self) -> DeResult<()> {
        if self.is_instant_dict_end {
            self.is_instant_dict_end = false;
            save_next_peek!(self, DeEvent::DictEnd);
        }
        // deserializer always throws an error if an Eof event occured
        if self.is_eof { panic!("Tried to read event after receiving EOF") }
        let mut expected = DeEventExpected::None;

        loop {
            match self.xml_next() {
                Ok(event) => {
                    match event {
                        XmlEvent::Decl(decl) => {
                            if let Ok(version) = decl.version() {
                                if let Ok(version) = String::from_utf8(version.to_vec()) {
                                    save_next_peek!(self, DeEvent::XmlVersion(version))
                                }
                            }
                            return Err(DeError::NoXmlVersionInfo);
                        }
                        XmlEvent::Start(tag) => {
                            if let DeEventExpected::None = expected {
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

                                        save_next_peek!(self, DeEvent::PlistStart {
                                            plist_version,
                                            gj_version
                                        })
                                    }
                                    b"d" | b"dict" => { save_next_peek!(self, DeEvent::DictStart) }
                                    b"k" => { expected = DeEventExpected::Key }
                                    b"s" => { expected = DeEventExpected::String }
                                    b"i" => { expected = DeEventExpected::Integer }
                                    b"r" => { expected = DeEventExpected::Real }
                                    b"t" => { save_next_peek!(self, DeEvent::True) }
                                    _ => { return Err(DeError::UnknownXmlTag) }
                                }
                            } else { return Err(DeError::UnexpectedXmlTag) }
                        }
                        XmlEvent::End(tag) => {
                            if let DeEventExpected::None = expected {
                                match tag.name().into_inner() {
                                    b"plist" | b"k" | b"s" | b"i" | b"r" | b"t" => {}
                                    b"d" | b"dict" => { save_next_peek!(self, DeEvent::DictEnd) }
                                    _ => { return Err(DeError::UnknownXmlTag) }
                                }
                            } else { return Err(DeError::UnexpectedXmlTag) }
                        }
                        XmlEvent::Empty(tag) => {
                            if let DeEventExpected::None = expected {
                                match tag.name().into_inner() {
                                    b"d" | b"dict" => {
                                        self.is_instant_dict_end = true;
                                        save_next_peek!(self, DeEvent::DictStart);
                                    }
                                    b"t" => { save_next_peek!(self, DeEvent::True) }
                                    _ => { return Err(DeError::UnknownXmlTag) }
                                }
                            } else { return Err(DeError::UnexpectedXmlTag); }
                        }
                        XmlEvent::Text(text) => {
                            if let DeEventExpected::None = expected { return Err(DeError::UnexpectedXmlText) }
                            else {
                                match text.unescape() {
                                    Ok(text) => match expected {
                                        DeEventExpected::None => { unreachable!() }
                                        DeEventExpected::Key => {
                                            save_next_peek!(self, DeEvent::Key(text.to_string()))
                                        }
                                        DeEventExpected::String => {
                                            save_next_peek!(self, DeEvent::String(text.to_string()))
                                        }
                                        DeEventExpected::Integer => {
                                            save_next_peek!(self, DeEvent::Integer(text.to_string()))
                                        }
                                        DeEventExpected::Real => {
                                            save_next_peek!(self, DeEvent::Real(text.to_string()))
                                        }
                                    }
                                    Err(err) => { return Err(DeError::XmlParse(err)) }
                                }
                            }
                        }
                        XmlEvent::Eof => {
                            self.is_eof = true;
                            save_next_peek!(self, DeEvent::Eof)
                        }
                        _ => { return Err(DeError::UnexpectedOtherXml) }
                    }
                }
                Err(error) => { return Err(DeError::XmlParse(error)) }
            }
        }
    }

    fn peek(&'a mut self) -> DeResult<&DeEvent> {
        if let None = self.peeked_next {
            self.save_next_peek()?;
        }
        if let Some(peeked) = &self.peeked_next {
            Ok(peeked)
        } else { unreachable!() }
    }
    
    fn next(&'a mut self) -> DeResult<Arc<DeEvent>> {
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
                DeEvent::String(text) |
                DeEvent::Key(text) |
                DeEvent::Integer(text) |
                DeEvent::Real(text) => {
                    if let Ok(parsed) = text.parse() { visitor.$visit(parsed) }
                    else { Err(DeError::Deserialization) }
                }
                DeEvent::True => { visitor.$visit($true) }
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

impl<'a, 'de, R: Read> de::MapAccess<'de> for DictReader<'a, 'de, R> {
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> DeResult<Option<K::Value>>
    where K: de::DeserializeSeed<'de> {
        match self.de.peek()? {
            DeEvent::DictEnd => Ok(None),
            DeEvent::Key(_) => Ok(Some(seed.deserialize(&mut *self.de)?)),
            _ => Err(DeError::Deserialization)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> DeResult<V::Value>
    where V: de::DeserializeSeed<'de> {
        match self.de.peek()? {
            DeEvent::DictStart |
            DeEvent::String(_) |
            DeEvent::Integer(_) |
            DeEvent::Real(_) |
            DeEvent::True => Ok(seed.deserialize(&mut *self.de)?),
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

impl<'a, 'de, R: Read> de::SeqAccess<'de> for ArrayReader<'a, 'de, R> {
    type Error = DeError;

    fn next_element_seed<T>(&mut self, seed: T) -> DeResult<Option<T::Value>>
    where T: de::DeserializeSeed<'de> {
        match &*self.de.next()? {
            DeEvent::DictEnd => Ok(None),
            DeEvent::Key(key) => {
                if *key == String::from("k_") + &self.cur_index.to_string() {
                    match self.de.peek()? {
                        DeEvent::DictStart |
                        DeEvent::String(_) |
                        DeEvent::Integer(_) |
                        DeEvent::Real(_) |
                        DeEvent::True => {self.cur_index+=1;Ok(Some(seed.deserialize(&mut *self.de)?))},
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
        if let DeEvent::XmlVersion(xml_version) = &*self.next()? {
            self.header.xml_version = xml_version.to_string();
            if let DeEvent::PlistStart { plist_version, gj_version } = &*self.next()? {
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
        if let DeEvent::DictEnd = *self.next().unwrap_or_else(|_| unreachable!()) { map }
        else { unreachable!() }
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<'de, R> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        match self.peek()? {
            DeEvent::DictStart => {
                self.next().unwrap_or_else(|_| unreachable!());
                match self.peek()? {
                    DeEvent::Key(key) => {
                        if key == "_isArr" {
                            self.next().unwrap_or_else(|_| unreachable!());
                            match *self.next()? {
                                DeEvent::True => visitor.visit_seq(ArrayReader::new(self)),
                                _ => Err(DeError::Deserialization)
                            }

                        } else {
                            self.deserialize_map_content(visitor)
                        }
                    }
                    DeEvent::DictEnd => self.deserialize_map_content(visitor),
                    _ => Err(DeError::Deserialization)
                }
            }
            DeEvent::String(_) => self.deserialize_str(visitor),
            DeEvent::Key(_) => self.deserialize_str(visitor),
            DeEvent::Integer(_) => self.deserialize_i32(visitor),
            DeEvent::Real(_) => self.deserialize_f32(visitor),
            DeEvent::True => self.deserialize_bool(visitor),
            _ => Err(DeError::Deserialization)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> DeResult<V::Value>
    where V: de::Visitor<'de> {
        if let DeEvent::True = *self.next()? {
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
            DeEvent::String(text) |
            DeEvent::Key(text) |
            DeEvent::Integer(text) |
            DeEvent::Real(text) => { visitor.visit_str(&text[..]) }
            DeEvent::True => { visitor.visit_str("true") }
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
        if let DeEvent::DictStart = *self.next()? {
            if let DeEvent::Key(key) = &*self.next()? {
                if key == "_isArr" {
                    match *self.next()? {
                        DeEvent::True => visitor.visit_seq(ArrayReader::new(self)),
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
        if let DeEvent::DictStart = *self.next()? {
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
        self.deserialize_any(visitor)
    }
}

#[cfg(test)]
mod tests;
