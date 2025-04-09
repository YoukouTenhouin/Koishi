use reqwest::{blocking::{self as req, RequestBuilder}, Url};
use serde::Deserialize;

pub use reqwest::Result;

use crate::global_options;

#[derive(Deserialize)]
pub(crate) struct ResponseSuccessWithData<T> {
    #[allow(dead_code)]
    success: bool,
    data: T,
}

#[derive(Deserialize)]
pub(crate) struct ResponseSuccess {
    #[allow(dead_code)]
    success: bool,
}

#[derive(Deserialize)]
pub(crate) struct ResponseError {
    #[serde(rename="error")]
    reason: String,
}

impl ResponseError {
    fn error_out(&self) -> ! {
	eprintln!("Server responded with error: {}", self.reason);
	std::process::exit(1)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum APIResponseWithData<T> {
    Ok(ResponseSuccessWithData<T>),
    Err(ResponseError),
}

impl<T> APIResponseWithData<T> {
    pub fn unwrap_or_error_out(self) -> T {
	match self {
	    APIResponseWithData::Ok(success_with_data) => {
		success_with_data.data
	    }
	    APIResponseWithData::Err(err) => {
		err.error_out()
	    }
	}
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum APIResponse {
    #[allow(dead_code)]
    Ok(ResponseSuccess),
    Err(ResponseError),
}

impl APIResponse {
    pub fn unwrap_or_error_out(self) {
	match self {
	    APIResponse::Ok(_) => {}
	    APIResponse::Err(err) => {
		err.error_out()
	    }
	}
    }
}

fn get_auth_key() -> String {
    let auth_key = global_options::AUTH_KEY.get().unwrap();
    let mut auth = "Bearer ".to_owned();
    auth.push_str(auth_key);
    auth
}

fn get_url<T: AsRef<str>>(path: T) -> Url {
    let url = global_options::BASE_URL.get().unwrap();
    let base = Url::parse(url).unwrap();
    base.join(path.as_ref()).unwrap()
}

pub(crate) fn get<T: AsRef<str>>(path: T) -> RequestBuilder {
    let url = get_url(path);
    req::Client::new()
        .get(url)
        .header("Authorization", get_auth_key())
}

pub(crate) fn post<T: AsRef<str>>(path: T) -> RequestBuilder {
    let url = get_url(path);
    req::Client::new()
        .post(url)
        .header("Authorization", get_auth_key())
}

pub(crate) fn put<T: AsRef<str>>(path: T) -> RequestBuilder {
    let url = get_url(path);
    req::Client::new()
        .put(url)
        .header("Authorization", get_auth_key())
}
