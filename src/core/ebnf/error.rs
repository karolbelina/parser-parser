use super::{builder, lexer, parser, scanner};
use js_sys::{Object, Reflect};
use std::{fmt, ops::Range};
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    position: Range<usize>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Scanner(scanner::error::Error),
    Lexer(lexer::error::Error),
    Parser(parser::error::Error),
    Builder(builder::error::Error),
}

impl From<scanner::error::Error> for Error {
    fn from(error: scanner::error::Error) -> Error {
        let position = error.position..error.position + 1;
        Error {
            kind: ErrorKind::Scanner(error),
            position,
        }
    }
}

impl From<lexer::error::Error> for Error {
    fn from(error: lexer::error::Error) -> Error {
        let position = error.position.clone();
        Error {
            kind: ErrorKind::Lexer(error),
            position,
        }
    }
}

impl From<parser::error::Error> for Error {
    fn from(error: parser::error::Error) -> Error {
        let position = error.position.clone();
        Error {
            kind: ErrorKind::Parser(error),
            position,
        }
    }
}

impl From<builder::error::Error> for Error {
    fn from(error: builder::error::Error) -> Error {
        let position = error.position.clone();
        Error {
            kind: ErrorKind::Builder(error),
            position,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Scanner(inner) => write!(f, "{}", inner),
            ErrorKind::Lexer(inner) => write!(f, "{}", inner),
            ErrorKind::Parser(inner) => write!(f, "{}", inner),
            ErrorKind::Builder(inner) => write!(f, "{}", inner),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Scanner(inner) => Some(inner),
            ErrorKind::Lexer(inner) => Some(inner),
            ErrorKind::Parser(inner) => Some(inner),
            ErrorKind::Builder(inner) => Some(inner),
        }
    }
}

#[allow(unused_unsafe)]
impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        let position = Object::new();
        unsafe {
            Reflect::set(
                &position,
                &"start".into(),
                &(self.position.start as u32).into(),
            )
            .unwrap();
            Reflect::set(&position, &"end".into(), &(self.position.end as u32).into()).unwrap();
        }
        let error = Object::new();
        unsafe {
            Reflect::set(&error, &"kind".into(), &self.to_string().into()).unwrap();
            Reflect::set(&error, &"position".into(), &position.into()).unwrap();
        }
        return error.into();
    }
}
