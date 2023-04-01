use std::fmt::{self, Display};
use serde::{ser, de};

use quick_xml::{
    Error as XmlError,
    events::attributes::AttrError as XmlAttrError
};

pub type DeResult<T> = std::result::Result<T, DeError>;

#[derive(Debug)]
pub enum DeError { // todo: precisify
    Open,
    Read,
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
        eprintln!("{}", msg);
        todo!();
    }
}

impl Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for DeError {}
