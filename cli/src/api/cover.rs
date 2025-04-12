use blake2::{Blake2b512, Digest};
use hex;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path, time::Duration};

use crate::global_options;
use crate::helpers::s3;

use super::request::{self, *};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize)]
struct Req {
    hash: String,
}

#[derive(Deserialize)]
struct ResCover {
    exists: bool,
    url: Option<String>,
}

pub struct UploadCoverResult {
    pub exists: bool,
    pub hash: String,
}

fn upload_url(hash: String) -> super::Result<ResCover> {
    let req_body = Req { hash };

    if global_options::DRY.get().unwrap().clone() {
        println!("skipping request due to being dry run");
        return Ok(ResCover {
            exists: true,
            url: None,
        });
    }

    request::post("cover").json(&req_body).send()?.api_result()
}

fn hash(content: &[u8]) -> String {
    let digest = Blake2b512::digest(content);
    hex::encode(digest)
}

pub(crate) fn upload_cover(content: Vec<u8>) -> Result<UploadCoverResult> {
    let hash = hash(content.as_slice());

    let res_url = upload_url(hash.clone())?;
    let ret = UploadCoverResult {
        exists: res_url.exists,
        hash,
    };
    if res_url.exists {
        return Ok(ret);
    }

    s3::Uploader::with_timeout(Duration::from_secs(300))?
        .url(res_url.url.unwrap())
        .mimetype("image/jpeg")
        .body(content)
        .upload()?;

    Ok(ret)
}

pub(crate) fn upload_cover_from_file<P: AsRef<Path>>(path: P) -> Result<UploadCoverResult> {
    let mut f = File::open(path)?;
    let f_size = f.metadata()?.len();
    let mut buf = vec![0; f_size as usize];
    f.read(&mut buf)?;

    upload_cover(buf)
}
