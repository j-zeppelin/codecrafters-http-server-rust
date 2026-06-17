use std::{
    collections::HashMap,
    fmt::{Display, format},
    io::BufRead,
    time::SystemTime,
};

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

#[derive(Debug)]
pub struct Request<'a> {
    pub line: RequestLine<'a>,
    pub headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub body: Option<String>,
}

impl<'a> Request<'a> {
    // TODO: rewrite this
    pub fn parse(mut reader: &mut impl BufRead) -> Result<Request<'a>, String> {
        let mut headers = HashMap::new();
        let mut body = String::new();

        let mut lines = reader.lines().peekable();

        if lines.peek().is_none() {
            return Err("empty request".to_string());
        };

        let request_line = RequestLine::parse(&lines.next().unwrap().unwrap())?;
        // headers
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

        while let Some(Ok(line)) = lines.next() {
            dbg!(&line);
            body.push_str(&line);
        }

        dbg!(headers);
        dbg!(body);

        todo!()

        // let mut headers: HashMap<String, &str> = HashMap::new();
        // let mut body_length = 0;
        //
        // while let Some(line) = lines.next() {
        //     if line.is_empty() {
        //         break;
        //     }
        //
        //     let (k, v) = line
        //         .split_once(':')
        //         .ok_or(format!("malformed header: {line}"))?;
        //
        //     let k = k.trim().to_lowercase();
        //
        //     if k == "content-length" {
        //         if let Ok(length) = v.trim().parse::<usize>() {
        //             body_length = length;
        //         }
        //     }
        //
        //     headers.insert(k, v.trim());
        // }
        //
        // let body = String::new();
        //
        // Ok(Self {
        //     line: request_line,
        //     headers,
        //     timestamp: SystemTime::now().into(),
        //     body: Some(body),
        // })
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
