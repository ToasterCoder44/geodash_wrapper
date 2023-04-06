use std::{
    error::Error,
    fmt::{self, Display},
    io::Error as IoError
};
use serde::{ser, de};

use quick_xml::{
    Error as XmlError,
    events::attributes::AttrError as XmlAttrError
};

pub type DeResult<T> = std::result::Result<T, DeError>;

#[derive(Debug)]
pub enum DeError {
    Custom(String),
    Io(IoError),
    XmlParse(XmlError),
    XmlAttrParse(XmlAttrError),
    NoXmlVersionInfo,
    UnexpectedOtherXml,
    UnexpectedXmlTag,
    UnexpectedXmlText,
    UnexpectedXmlAttr,
    UnknownXmlTag,
    ExpectedXmlVersion,
    ExpectedPlistVersion,
    ExpectedGjVersion,
    ExpectedEof,
    Deserialization
}

impl de::Error for DeError {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

impl Error for DeError {}
