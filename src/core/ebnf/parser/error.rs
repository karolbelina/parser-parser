use super::TokenKind;
use nom::error::ErrorKind;
use nom::error::ParseError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnterminatedSpecial,
    UnterminatedComment,
    UnterminatedTerminal,
    EmptyTerminal,
    NonterminalExpected,
    TerminalExpected,
    SpecialExpected,
    IntegerExpected,
    InvalidToken,
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_: I, _: ErrorKind) -> Self {
        Error::InvalidToken
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnterminatedSpecial => write!(f, "unterminated special sequence"),
            Error::UnterminatedComment => write!(f, "unterminated comment"),
            Error::UnterminatedTerminal => write!(f, "unterminated terminal symbol"),
            Error::EmptyTerminal => write!(f, "empty terminal symbol"),
            Error::NonterminalExpected => write!(f, "nonterminal expected"),
            Error::TerminalExpected => write!(f, "terminal expected"),
            Error::SpecialExpected => write!(f, "special sequence expected"),
            Error::IntegerExpected => write!(f, "integer expected"),
            Error::InvalidToken => write!(f, "invalid token"),
        }
    }
}

impl std::error::Error for Error {}
