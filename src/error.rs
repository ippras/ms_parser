use nom::{
    error::{ErrorKind, FromExternalError, ParseError},
    Err,
};
use std::fmt::Debug;

// #[derive(Debug, PartialEq)]
// pub enum CustomError<I> {
//     MyError,
//     Nom(I, ErrorKind),
// }

/// Result
pub type Result<T, E = Err<Error>> = std::result::Result<T, E>;

// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("Nom error {{ input: {input:?}, kind: {kind:?} }}")]
//     Nom { input: Vec<u8>, kind: ErrorKind },
//     #[error("From io error")]
//     Io(#[from] io::Error),
// }

/// Error
#[derive(Debug)]
pub struct Error {
    pub input: Vec<u8>,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(input: &[u8], kind: ErrorKind) -> Error {
        Self {
            input: input.to_owned(),
            kind,
        }
    }
}

impl<E> FromExternalError<&[u8], E> for Error {
    fn from_external_error(input: &[u8], kind: ErrorKind, _: E) -> Self {
        Self::new(input, kind)
    }
}

impl ParseError<&[u8]> for Error {
    fn from_error_kind(input: &[u8], kind: ErrorKind) -> Self {
        Self::new(input, kind)
    }

    fn append(_: &[u8], _: ErrorKind, other: Self) -> Self {
        other
    }
}

// impl<I: Debug> Debug for Error<I> {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         f.debug_struct("Error")
//             .field("input", &self.input)
//             .field("kind", &self.kind)
//             .finish()
//     }
// }

// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("From anyhow error")]
//     Anyhow(#[from] Err<anyhow::Error>),
//     #[error("From io error")]
//     Io(#[from] io::Error),
//     #[error("From nom error")]
//     Nom(#[from] Err<nom::error::Error<Vec<u8>>>),
//     #[error("From utf8 error")]
//     Utf8(#[from] Utf8Error),

//     // #[error("data store disconnected")]
//     // Nom1(#[from] nom::error::Error<Vec<u8>>),
//     #[error("the data for key `{0}` is not available")]
//     Redaction(String),
//     #[error("invalid header (expected {expected:?}, found {found:?})")]
//     InvalidHeader { expected: String, found: String },
//     #[error("unknown data store error")]
//     Unknown,
// }

// impl FromExternalError<&[u8], Utf8Error> for Error {
//     fn from_external_error(input: &[u8], kind: ErrorKind, error: Utf8Error) -> Self {
//         make_error(input, kind)
//     }
// }

// impl ParseError<&[u8]> for Error {
//     fn from_error_kind(input: &[u8], kind: ErrorKind) -> Self {
//         Error::Nom(Err::Error(nom::error::Error::from_error_kind(input, kind)).to_owned())
//     }

//     fn append(_: &[u8], _: ErrorKind, other: Self) -> Self {
//         other
//     }
// }

// impl From<Err<nom::error::Error<&[u8]>>> for Err<Error> {
//     fn from(value: Err<nom::error::Error<&[u8]>>) -> Self {
//         Err::Error(Self::Nom(value.to_owned()))
//     }
// }
