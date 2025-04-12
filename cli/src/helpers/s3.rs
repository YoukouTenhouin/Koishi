use reqwest::{
    IntoUrl, Result as ReqResult,
    blocking::{Body, Client, RequestBuilder, Response},
};
use serde::Deserialize;
use std::{
    fmt::Display,
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
    time::Duration,
};

#[derive(Debug)]
pub(crate) struct S3Error {
    status: u16,
    xml: String,
}

impl Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S3Error {}", self.status)?;
        if f.alternate() {
            write!(f, "\n{}\n", self.xml)?;
        }
        Ok(())
    }
}

impl std::error::Error for S3Error {}

#[derive(Debug)]
pub(crate) enum S3UploaderError {
    S3(S3Error),
    Request(reqwest::Error),
    IO(io::Error),
}

impl Display for S3UploaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:?}", self)
        } else {
            write!(f, "{:#?}", self)
        }
    }
}

impl std::error::Error for S3UploaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            S3UploaderError::S3(err) => Some(err),
            S3UploaderError::Request(err) => Some(err),
            S3UploaderError::IO(err) => Some(err),
        }
    }
}

impl From<S3Error> for S3UploaderError {
    fn from(value: S3Error) -> Self {
        S3UploaderError::S3(value)
    }
}

impl From<reqwest::Error> for S3UploaderError {
    fn from(value: reqwest::Error) -> Self {
        S3UploaderError::Request(value)
    }
}

impl From<io::Error> for S3UploaderError {
    fn from(value: io::Error) -> Self {
        S3UploaderError::IO(value)
    }
}

pub(crate) struct Uploader {
    client: Client,
}

impl Uploader {
    pub fn new() -> ReqResult<Self> {
        Self::with_timeout(None)
    }

    pub fn with_timeout<T: Into<Option<Duration>>>(timeout: T) -> ReqResult<Self> {
        let client = Client::builder().timeout(timeout).build()?;
        Ok(Self { client })
    }

    pub fn url<U: IntoUrl>(&self, url: U) -> UploadTaskBuilder {
        let rb = self.client.put(url);
        UploadTaskBuilder { rb }
    }
}

pub(crate) struct UploadTaskBuilder {
    rb: RequestBuilder,
}

impl UploadTaskBuilder {
    fn map_inner<F>(self, f: F) -> Self
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        let rb = f(self.rb);
        Self { rb }
    }

    pub fn mimetype<S: AsRef<str>>(self, value: S) -> Self {
        self.map_inner(|rb| rb.header("Content-Type", value.as_ref()))
    }

    pub fn copy<S: AsRef<str>>(self, source: S) -> Self {
        self.map_inner(|rb| rb.header("x-amz-copy-source", source.as_ref()))
    }

    pub fn copy_range_from_to(self, from: u64, to: u64) -> Self {
        self.map_inner(|rb| rb.header("x-amz-copy-source-range", format!("bytes={}-{}", from, to)))
    }

    pub fn body<B: Into<Body>>(self, body: B) -> Self {
        self.map_inner(|rb| rb.body(body))
    }

    pub fn from_reader_sized<R: Read + Send + 'static>(self, reader: R, limit: u64) -> Self {
        self.body(Body::sized(reader.take(limit), limit))
    }

    pub fn from_file_path<P: AsRef<Path>>(self, path: P) -> io::Result<Self> {
        let f = File::open(path)?;
        let f_size = f.metadata()?.len();

        let buf_reader = BufReader::new(f);

        Ok(self.from_reader_sized(buf_reader, f_size as u64))
    }

    fn send(self) -> Result<Response, S3UploaderError> {
        let res = self.rb.send()?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let xml = res.text()?;
            Err(S3Error { status, xml }.into())
        } else {
            Ok(res)
        }
    }

    pub fn upload(self) -> Result<UploadResult, S3UploaderError> {
        let res = self.send()?;
        let etag_header = res
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());
        if let Some(etag) = etag_header {
            Ok(UploadResult { etag })
        } else {
            let xml = res.text()?;
            let result: CopyPartResult =
                quick_xml::de::from_str(&xml).expect("Failed to parse XML");
            Ok(UploadResult { etag: result.etag })
        }
    }
}

pub(crate) struct UploadResult {
    pub etag: String,
}

#[derive(Deserialize)]
struct CopyPartResult {
    #[serde(rename = "ETag")]
    etag: String,
}
