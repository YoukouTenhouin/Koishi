use serde::Deserialize;
use serde_json::Value as JsonValue;

mod request;
pub mod cover;
pub mod room;
pub mod video;

#[derive(Deserialize)]
pub(crate) struct ServerError {
    #[serde(rename="error")]
    pub error_type: ServerErrorType,
    pub message: Option<String>,
    pub details: Option<JsonValue>
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.error_type)?;
	if let Some(msg) = self.message.as_ref(){
	    write!(f, ": {}", msg)?;
	};
	if f.alternate() {
	    if let Some(details) = self.details.as_ref() {
		write!(f, "\n{:#}\n", details)?;
	    }
	}
	Ok(())
    }
}

impl std::fmt::Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{:#}", self)
    }
}

impl std::error::Error for ServerError {}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all="snake_case")]
pub(crate) enum ServerErrorType {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    Conflict,
    UnprocessableEntity,
    InternalServerError,

    DBTransactionError,
    #[serde(rename="s3_error")]
    S3Error,
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for ServerErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	match self {
	    Self::BadRequest => write!(f, "BadRequest"),
	    Self::Unauthorized => write!(f, "Unauthorized"),
	    Self::Forbidden => write!(f, "Forbidden"),
	    Self::NotFound => write!(f, "NotFound"),
	    Self::MethodNotAllowed => write!(f, "MethodNotAllowed"),
	    Self::Conflict => write!(f, "Conflict"),
	    Self::UnprocessableEntity => write!(f, "UnprocessableEntity"),
	    Self::InternalServerError => write!(f, "InternalServerError"),

	    Self::DBTransactionError => write!(f, "DBTransactionError"),
	    Self::S3Error => write!(f, "S3Error"),
	    Self::Unknown(type_str) => write!(f, "UnknownError({})", type_str),
	}
    }
}

pub(crate) type Result<T> = std::result::Result<T, APIError>;

pub(crate) enum APIError {
    ServerError(ServerError),
    RequestError(reqwest::Error),
}

impl From<reqwest::Error> for APIError {
    fn from(value: reqwest::Error) -> Self {
	APIError::RequestError(value)
    }
}

impl From<ServerError> for APIError {
    fn from(value: ServerError) -> Self {
	APIError::ServerError(value)
    }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	match self {
	    Self::ServerError(err) => {
		if f.alternate() {
		    write!(f, "Server returned error: {:#}", err)
		} else {
		    write!(f, "ServerError: {}", err)
		}
	    },
	    Self::RequestError(err) => {
		if f.alternate() {
		    write!(f, "Error during request: {:#}", err)
		} else {
		    write!(f, "RequestError: {}", err)
		}
	    }
	}
    }
}

impl std::fmt::Debug for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	match self {
	    Self::ServerError(err) => {
		write!(f, "Server returned error: {:#?}", err)
	    },
	    Self::RequestError(err) => {
		write!(f, "Error during request: ")?;
		if f.alternate() {
		    write!(f, "{:#?}", err)
		} else {
		    write!(f, "{:?}", err)
		}
	    }
	}
    }
}

impl std::error::Error for APIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
	match self {
	    Self::ServerError(err) => Some(err),
	    Self::RequestError(err) => Some(err)
	}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::from_str;

    #[test]
    fn test_server_error_format() {
	assert_eq!(
	    format!("{}", ServerErrorType::NotFound),
	    "NotFound"
	);

	assert_eq!(
	    format!("{}", ServerErrorType::Unknown("unknown_error".into())),
	    "UnknownError(unknown_error)"
	);
    }

    #[test]
    fn test_server_error_deserialize() {
	let value = from_str::<ServerErrorType>("\"bad_request\"").unwrap();
	assert_eq!(value, ServerErrorType::BadRequest);

	let value = from_str::<ServerErrorType>("\"some_random_error\"").unwrap();
	assert_eq!(value, ServerErrorType::Unknown("some_random_error".into()));
    }
}
