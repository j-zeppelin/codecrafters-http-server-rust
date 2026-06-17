use std::{
    collections::HashMap,
    fmt::{Display, format},
    io::BufRead,
    time::SystemTime,
};

use chrono::{DateTime, Utc};

use crate::server::http::HttpMethod;

#[derive(Debug)]
pub struct RequestLine {
    pub method: HttpMethod,
    pub target: String,
    pub http_version: String,
}

impl RequestLine {
    fn parse(line: &str) -> Result<RequestLine, String> {
        let parts: Vec<_> = line.split_whitespace().collect();

        if parts.len() != 3 {
            return Err("invalid request line".to_string());
        }

        let method = HttpMethod::try_from(parts[0])?;
        let target = parts[1].to_string();
        let http_version = parts[2].to_string();

        Ok(Self {
            method,
            target,
            http_version,
        })
    }
}

impl Display for RequestLine {
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

#[derive(Debug)]
pub struct Request {
    pub line: RequestLine,
    pub headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub body: Option<String>,
}

impl Request {
    // TODO: rewrite this
    pub fn parse(reader: &mut impl BufRead) -> Result<Self, String> {
        let mut headers = HashMap::new();
        let mut lines = reader.lines().peekable();

        if lines.peek().is_none() {
            return Err("empty request".to_string());
        };

        let request_line = RequestLine::parse(lines.next().unwrap().unwrap().as_str())?;

        // parse headers
        while let Some(Ok(line)) = lines.next() {
            if line.is_empty() {
                break;
            }

            let (k, v) = line
                .split_once(':')
                .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
                .ok_or(format!("malformed header: {line}"))?;

            headers.insert(k, v);
        }

        // parse body
        let body: Option<String> = if let Some(content_length) = headers.get("content-length") {
            if let Ok(length) = content_length.parse::<usize>() {
                let mut buf = vec![0; length];
                reader
                    .read_exact(&mut buf)
                    .map_err(|e| format!("could not read body: {e}"))?;
                Some(
                    std::str::from_utf8(&buf)
                        .map_err(|e| format!("invalid UTF-8: {e}"))?
                        .to_string(),
                )
            } else {
                return Err("Invalid content length".to_string());
            }
        } else {
            None
        };

        Ok(Self {
            line: request_line,
            headers,
            timestamp: SystemTime::now().into(),
            body,
        })
    }
}

impl Display for Request {
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
