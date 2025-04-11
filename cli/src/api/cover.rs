use serde::{Deserialize, Serialize};

use crate::global_options;

use super::{Result, request::{self, *}};

#[derive(Serialize)]
struct Req {
    hash: String,
}

#[derive(Deserialize)]
pub(crate) struct CoverUploadResponse {
    pub exists: bool,
    pub url: Option<String>,
}

pub(crate) fn upload_url(hash: String) -> Result<CoverUploadResponse> {
    let req_body = Req { hash };

    if global_options::DRY.get().unwrap().clone() {
	println!("skipping request due to being dry run");
	return Ok(CoverUploadResponse{ exists: true, url: None })
    }

    request::post("cover")
        .json(&req_body)
        .send()?
	.api_result()
}
