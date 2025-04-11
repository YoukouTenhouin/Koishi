use serde::{de::DeserializeOwned, Deserialize};
use std::any::TypeId;
use reqwest::{blocking::{self as req, RequestBuilder, Response}, Url};

use crate::global_options;

use super::{Result, ServerError};

#[derive(Deserialize)]
pub(crate) struct APISuccess<T> {
    #[allow(dead_code)]
    success: bool,
    data: Option<T>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum APIRawResult<T> {
    Success(APISuccess<T>),
    Error(ServerError),
}

fn get_auth_key() -> Option<String> {
    global_options::AUTH_KEY.get()
        .unwrap()
        .as_ref()
	.map(|s| {
	    let mut auth = "Bearer ".to_owned();
	    auth.push_str(s.as_str());
	    auth
	})
}

fn concat_api_url<P: AsRef<str>>(path: P) -> Url {
    let url = global_options::BASE_URL.get().unwrap();
    let base = Url::parse(url).unwrap();
    base.join(path.as_ref()).unwrap()
}

pub(super) fn get<P: AsRef<str>>(path: P) -> RequestBuilder {
    let url = concat_api_url(path);
    req::Client::new()
        .get(url)
}

pub(super) fn post<P: AsRef<str>>(path: P) -> RequestBuilder {
    let url = concat_api_url(path);
    req::Client::new()
        .post(url)
        .api_auth()
}

pub(super) fn put<P: AsRef<str>>(path: P) -> RequestBuilder {
    let url = concat_api_url(path);
    req::Client::new()
        .put(url)
        .api_auth()
}

pub(super) trait APIRequestBuilder {
    fn api_auth(self) -> Self;
    fn limit_offset(self, limit: u64, offset: u64) -> Self;
}

impl APIRequestBuilder for RequestBuilder {
    fn api_auth(self) -> Self {
	self.header(
	    "Authorization",
	    get_auth_key().expect("Authorization key required")
	)
    }

    fn limit_offset(self, limit: u64, offset: u64) -> Self {
	self.query(&[
	    ("limit", limit.to_string()),
	    ("offset", offset.to_string())
	])
    }
}

pub(super) trait APIResult {
    fn api_result<T: DeserializeOwned + 'static>(self) -> Result<T>;
}

impl APIResult for Response {
    fn api_result<T: DeserializeOwned + 'static>(self) -> Result<T> {
	let raw: APIRawResult<T> = self.json()?;
	match raw {
	    APIRawResult::Success(success) => {
		if TypeId::of::<T>() == TypeId::of::<()>() {
		    Ok(serde_json::from_value::<T>(serde_json::Value::Null).unwrap())
		} else {
		    let data = success.data.expect("Server should send an 'data' field; got none");
		    Ok(data)
		}
	    }
	    APIRawResult::Error(err) => Err(err.into()),
	}
    }
}
