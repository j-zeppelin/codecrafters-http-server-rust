use std::{fmt::Display, marker::PhantomData};

use crate::server::http::{HttpStatus, HttpVersion};

pub struct Missing;
pub struct Present;

pub struct ResponseBuilder<Status> {
    version: Option<HttpVersion>,
    status: Option<HttpStatus>,
    headers: Vec<(String, String)>,
    body: Option<String>,
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
        for (k, v) in values {
            self.headers.push((k.into(), v.into()));
        }
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
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

pub struct Response {
    pub version: HttpVersion,
    pub status: HttpStatus,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl Response {
    pub fn builder() -> ResponseBuilder<Missing> {
        ResponseBuilder::new()
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers: String = self
            .headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}\r\n"))
            .collect();

        let result = format!(
            "{} {}\r\n{}\r\n{}",
            self.version.as_str(),
            self.status.as_str(),
            headers,
            self.body.as_ref().map_or("", |v| v)
        );

        write!(f, "{result}")
    }
}
