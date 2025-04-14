use serde::{Deserialize, Serialize};
use std::{error, path::Path, result, time::Duration};
use tabled::{Tabled, derive::display};

use crate::global_options;
use crate::helpers::{self, s3, se::BoolAsInt};

use super::{
    Result,
    request::{self, *},
};

#[derive(Serialize, Deserialize, Tabled)]
pub(crate) struct Video {
    #[tabled(rename = "UUID")]
    pub uuid: String,
    #[tabled(rename = "Title")]
    pub title: String,
    #[tabled(rename = "Cover URL", display("display::option", "<Not set>"))]
    pub cover: Option<String>,
    #[tabled(rename = "Room ID")]
    pub room: u64,
    #[tabled(rename = "Restricted")]
    pub restricted: BoolAsInt,
    #[tabled(rename = "Date", display("helpers::tabled::timestamp", self))]
    pub timestamp: i64,
}

#[derive(Serialize)]
struct VideoCreateInfo {
    title: String,
    cover: Option<String>,
    timestamp: i64,
    room: u64,
    restricted: BoolAsInt,
    restricted_hash: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct MetadataUploadResponse {
    pub url: String,
}

#[derive(Deserialize)]
pub(crate) struct VideoUploadStartResponse {
    pub urls: Vec<String>,
    pub upload_id: String,
    pub video: Video,
}

#[derive(Deserialize)]
pub(crate) struct VideoSetRestrictedResponse {
    pub copy_source: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct RestrictedCopyStartResponse {
    pub length: u64,
    pub upload_id: String,
    pub urls: Vec<String>,
}

#[derive(Serialize)]
struct VideoUpdateInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
}

#[derive(Serialize)]
struct ReqUploadStart {
    size: u64,
    part_size: u64,
    restricted_hash: Option<String>,
}

#[derive(Serialize)]
struct ReqUploadFinish {
    upload_id: String,
    etags: Vec<String>,
    restricted_hash: Option<String>,
}

#[derive(Serialize)]
struct ReqSetRestricted {
    restricted: u64,
    hash: String,
}

#[derive(Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
enum ReqPostRestricted {
    CopyStart {
        copy_source: String,
        hash: Option<String>,
        part_size: u64,
    },
    CopyFinish {
        copy_source: String,
        hash: Option<String>,
        etags: Vec<String>,
        upload_id: String,
    },
}

pub(crate) fn create(
    uuid: &str,
    title: String,
    cover: Option<String>,
    timestamp: i64,
    room: u64,
    restricted_hash: Option<String>,
) -> Result<()> {
    if global_options::DRY.get().unwrap().clone() {
        println!("skipping request due to being dry run");
        return Ok(());
    }

    let restricted: BoolAsInt = restricted_hash.is_some().into();

    let video = VideoCreateInfo {
        title,
        cover,
        room,
        timestamp,
        restricted,
        restricted_hash,
    };

    request::post(format!("video/{uuid}"))
        .json(&video)
        .send()?
        .api_result()
}

pub(crate) fn update(
    uuid: &str,
    title: Option<String>,
    cover: Option<String>,
    timestamp: Option<i64>,
) -> Result<()> {
    if global_options::DRY.get().unwrap().clone() {
        println!("skipping request due to being dry run");
        return Ok(());
    }

    let video = VideoUpdateInfo {
        title,
        cover,
        timestamp,
    };

    request::put(format!("video/{uuid}"))
        .json(&video)
        .send()?
        .api_result()
}

fn metadata_upload_url(uuid: &str) -> Result<String> {
    if global_options::DRY.get().unwrap().clone() {
        println!("skipping request due to being dry run");
        return Ok("".into());
    }

    let res: MetadataUploadResponse = request::post(format!("video/{uuid}/upload_metadata"))
        .send()?
        .api_result()?;

    Ok(res.url)
}

pub(crate) fn upload_metadata<P: AsRef<Path>>(
    uuid: &str,
    path: P,
) -> result::Result<(), Box<dyn error::Error>> {
    let url = metadata_upload_url(uuid)?;

    if global_options::DRY.get().unwrap().clone() {
        return Ok(());
    }

    s3::Uploader::with_timeout(Duration::from_secs(300))?
        .url(url)
        .from_file_path(path)?
        .upload()?;

    Ok(())
}

pub(crate) fn upload_start(
    uuid: &str,
    file_size: u64,
    part_size: u64,
    hash: Option<String>,
) -> Result<VideoUploadStartResponse> {
    let req_body = ReqUploadStart {
        size: file_size,
        part_size,
        restricted_hash: hash,
    };
    request::post(format!("video/{uuid}/upload_start"))
        .json(&req_body)
        .send()?
        .api_result()
}

pub(crate) fn upload_finish(
    uuid: &str,
    upload_id: String,
    etags: Vec<String>,
    hash: Option<String>,
) -> Result<()> {
    let req_body = ReqUploadFinish {
        upload_id,
        etags,
        restricted_hash: hash,
    };

    request::post(format!("video/{uuid}/upload_finish"))
        .json(&req_body)
        .send()?
        .api_result()
}

pub(crate) fn set_restricted(
    uuid: &str,
    restricted: bool,
    hash: &str,
) -> Result<VideoSetRestrictedResponse> {
    let req_body = ReqSetRestricted {
        restricted: restricted as u64,
        hash: hash.to_string(),
    };

    request::put(format!("video/{uuid}/restricted"))
        .json(&req_body)
        .send()?
        .api_result()
}

pub(crate) fn restricted_copy_start(
    uuid: &str,
    copy_source: &str,
    hash: Option<String>,
    part_size: u64,
) -> Result<RestrictedCopyStartResponse> {
    let copy_source = copy_source.to_string();
    let req_body = ReqPostRestricted::CopyStart {
        copy_source,
        hash,
        part_size,
    };

    request::post(format!("video/{uuid}/restricted"))
        .json(&req_body)
        .send()?
        .api_result()
}

pub(crate) fn restricted_copy_finish(
    uuid: &str,
    copy_source: &str,
    hash: Option<String>,
    upload_id: &str,
    etags: Vec<String>,
) -> Result<()> {
    let copy_source =copy_source.to_string();
    let upload_id = upload_id.to_string();
    let req_body = ReqPostRestricted::CopyFinish {
        copy_source,
        hash,
        etags,
        upload_id,
    };

    request::post(format!("video/{uuid}/restricted"))
        .json(&req_body)
        .send()?
        .api_result()
}
