use iron::prelude::*;
use iron::status;
use serde_json::error::Error as JsonError;
use std::error::Error as StdError;
use std::fmt::Error as FmtError;
use std::fmt::{Display, Formatter};

/// A custom type that is ultimately used to convert client request errors and internal server
/// errors to appropriate HTTP responses.
#[derive(Debug)]
pub enum Error {
    /// A requested resource was not found.
    NotFound,
    /// An error regarding a request's parameters.
    BadRequest,
    /// A JSON serialization error.
    JsonError(JsonError),
}

impl Error {
    /// Consume the error to produce an `IronError` appropriate for returning from an Iron request
    /// handler.
    pub fn into_iron_result(self) -> IronResult<Response> {
        Err(IronError::from(self))
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NotFound => "Resource not found",
            Error::BadRequest => "Bad request",
            Error::JsonError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::JsonError(ref err) => err.cause(),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match *self {
            Error::JsonError(ref err) => Display::fmt(err, f),
            _ => self.description().fmt(f),
        }
    }
}

impl From<Error> for IronError {
    fn from(err: Error) -> IronError {
        let mut response = Response::new();

        match err {
            Error::BadRequest => { response.set_mut(status::BadRequest); },
            Error::NotFound => { response.set_mut(status::NotFound); },
            _ => { response.set_mut(status::InternalServerError); },
        }

        IronError { error: Box::new(err), response: response }
    }
}
