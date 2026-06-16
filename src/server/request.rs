use std::{collections::HashMap, fmt::Display, time::SystemTime};

use chrono::{DateTime, Utc};

use crate::server::http::HttpMethod;

#[derive(Debug)]
pub struct RequestLine<'a> {
    pub method: HttpMethod,
    pub target: &'a str,
    pub http_version: &'a str,
}

impl<'a> RequestLine<'a> {
    fn parse(line: &'a str) -> Result<RequestLine<'a>, String> {
        let parts: Vec<_> = line.split_whitespace().collect();

        let method = HttpMethod::try_from(parts[0])?;
        let target = parts[1];
        let http_version = parts[2];

        Ok(Self {
            method,
            target,
            http_version,
        })
    }
}

impl Display for RequestLine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.method.as_str(),
            self.target,
            self.http_version
        )
    }
}

pub struct Request<'a> {
    pub line: RequestLine<'a>,
    pub headers: HashMap<String, &'a str>,
    pub timestamp: DateTime<Utc>,
}

impl<'a> Request<'a> {
    pub fn parse(buf: &'a str) -> Result<Request<'a>, String> {
        let mut lines = buf.split("\r\n").peekable();
        if lines.peek().is_none() {
            return Err("empty request".to_string());
        }

        let request_line = RequestLine::parse(lines.next().unwrap())?;

        let mut headers: HashMap<String, &str> = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }

            let (k, v) = line
                .split_once(':')
                .ok_or(format!("malformed header: {line}"))?;

            let k = k.trim().to_lowercase();
            headers.insert(k, v.trim());
        }

        Ok(Self {
            line: request_line,
            headers,
            timestamp: SystemTime::now().into(),
        })
    }
}

impl Display for Request<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.line.method.as_str(),
            self.line.target,
            self.line.http_version
        )
    }
}
