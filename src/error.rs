//! Errors, errors, errors

use std::io::Error as IoError;
use std::rc::Rc;

use plist::Error as PlistError;
use quick_xml::Error as XmlError;

/// Errors that occur while working with font objects.
#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    ParseError(XmlError),
    ParseGlif(ParseGlifError),
    MissingFile(&'static str),
    PlistError(PlistError),
    MissingGlyph,
    /// A wrapper for stashing errors for later use.
    SavedError(Rc<Error>),
}

#[doc(hidden)]
impl From<XmlError> for Error {
    fn from(src: XmlError) -> Error {
        Error::ParseError(src)
    }
}

#[doc(hidden)]
impl From<PlistError> for Error {
    fn from(src: PlistError) -> Error {
        Error::PlistError(src)
    }
}

#[doc(hidden)]
impl From<IoError> for Error {
    fn from(src: IoError) -> Error {
        Error::IoError(src)
    }
}

/// The location of a `.glif` parse failure, and the reported reason.
#[derive(Debug, Clone)]
pub struct ParseGlifError {
    pub kind: ErrorKind,
    pub position: usize,
}

impl ParseGlifError {
    pub fn new(kind: ErrorKind, position: usize) -> Self {
        ParseGlifError { kind, position }
    }
}

/// The reason for a glif parse failure.
#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnsupportedGlifVersion,
    UnknownPointType,
    WrongFirstElement,
    MissingCloseTag,
    UnexpectedTag,
    BadHexValue,
    BadNumber,
    BadColor,
    BadAnchor,
    BadPoint,
    BadGuideline,
    BadComponent,
    BadImage,
    UnexpectedDuplicate,
    UnexpectedElement,
    UnexpectedEof,
}

impl ErrorKind {
    pub(crate) fn to_error(self, position: usize) -> ParseGlifError {
        ParseGlifError { kind: self, position }
    }
}

#[doc(hidden)]
impl From<ParseGlifError> for Error {
    fn from(src: ParseGlifError) -> Error {
        Error::ParseGlif(src)
    }
}
