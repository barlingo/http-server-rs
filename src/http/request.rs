use super::QueryString;
use super::{Method, MethodError};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::{self, Utf8Error};

// TODO: increase request and response implementation
// TODO: increase add multiple requests.

#[derive(Debug)]
pub struct Request<'buf> {
    path: &'buf str,
    query_str: Option<QueryString<'buf>>,
    method: Method,
}
impl<'buf> Request<'buf> {
    pub fn path(&self) -> &str {
        self.path
    }
    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn query_str(&self) -> Option<&QueryString> {
        self.query_str.as_ref()
    }
}

// TryFrom implements TryInto for free
impl<'buf> TryFrom<&'buf [u8]> for Request<'buf> {
    type Error = ParseErrorInvalid;
    fn try_from(buf: &'buf [u8]) -> Result<Self, Self::Error> {
        let request = str::from_utf8(buf)?;

        let (method, request) = get_next_word(request).ok_or(ParseErrorInvalid::Request)?;
        let (mut path, request) = get_next_word(request).ok_or(ParseErrorInvalid::Request)?;
        let (protocol, _) = get_next_word(request).ok_or(ParseErrorInvalid::Request)?;

        if protocol != "HTTP/1.1" {
            return Err(ParseErrorInvalid::Protocol);
        }
        let method: Method = method.parse()?;

        let mut query_str = None;
        if let Some(i) = path.find('?') {
            query_str = Some(QueryString::from(&path[i + 1..]));
            path = &path[..i];
        }
        Ok(Self {
            path,
            query_str,
            method,
        })
    }
}

fn get_next_word(request: &str) -> Option<(&str, &str)> {
    for (i, c) in request.chars().enumerate() {
        if c == ' ' || c == '\r' {
            return Some((&request[..i], &request[i + 1..]));
        }
    }
    None
}

pub enum ParseErrorInvalid {
    Request,
    Encoding,
    Protocol,
    Method,
}

impl ParseErrorInvalid {
    fn message(&self) -> &str {
        match self {
            Self::Request => "Invalid Request",
            Self::Encoding => "Invalid Encoding",
            Self::Protocol => "Invalid Protocol",
            Self::Method => "Invalid Method",
        }
    }
}

impl From<Utf8Error> for ParseErrorInvalid {
    fn from(_: Utf8Error) -> Self {
        Self::Encoding
    }
}

impl From<MethodError> for ParseErrorInvalid {
    fn from(_: MethodError) -> Self {
        Self::Method
    }
}
impl Debug for ParseErrorInvalid {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Display for ParseErrorInvalid {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ParseErrorInvalid {}
