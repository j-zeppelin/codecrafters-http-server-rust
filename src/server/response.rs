use std::{io::Write, marker::PhantomData};

use anyhow::Result;
use flate2::{Compression, write::GzEncoder};

use crate::server::http::{HttpStatus, HttpVersion};

pub struct Missing;
pub struct Present;

pub struct ResponseBuilder<Status> {
    version: Option<HttpVersion>,
    status: Option<HttpStatus>,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    _status: PhantomData<Status>,
}

impl ResponseBuilder<Missing> {
    fn new() -> Self {
        Self {
            version: None,
            status: None,
            headers: vec![],
            body: None,
            _status: PhantomData,
        }
    }
}

impl<Status> ResponseBuilder<Status> {
    #[allow(dead_code)]
    pub fn version(mut self, version: HttpVersion) -> Self {
        self.version = Some(version);
        self
    }

    pub fn status(self, status: HttpStatus) -> ResponseBuilder<Present> {
        ResponseBuilder {
            version: self.version,
            status: Some(status),
            headers: self.headers,
            body: self.body,
            _status: PhantomData,
        }
    }

    pub fn headers<T: Into<String>>(mut self, values: Vec<(T, T)>) -> Self {
        self.headers = values
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

impl ResponseBuilder<Present> {
    pub fn build(self) -> Response {
        Response {
            version: self.version.unwrap_or(HttpVersion::Http11),
            status: self.status.unwrap(),
            headers: self.headers,
            body: self.body,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub version: HttpVersion,
    pub status: HttpStatus,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    pub fn builder() -> ResponseBuilder<Missing> {
        ResponseBuilder::new()
    }

    pub fn compress_body(&mut self) -> Result<Vec<u8>, String> {
        if let Some(body) = &self.body {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(body)
                .map_err(|e| format!("could not encode body: {e}"))?;
            return encoder.finish().map_err(|e| e.to_string());
        }
        Err("no body to compress".to_string())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let headers: String = self
            .headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}\r\n"))
            .collect();

        let status_line = format!("{} {}\r\n", self.version.as_str(), self.status.as_str());

        let mut bytes = Vec::new();
        bytes.extend_from_slice(status_line.as_bytes());
        bytes.extend_from_slice(headers.as_bytes());
        bytes.extend_from_slice(b"\r\n");
        if let Some(body) = &self.body {
            bytes.extend_from_slice(body);
        }
        bytes
    }
}
