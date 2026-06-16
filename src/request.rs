pub enum HttpMethod {
    GET,
    POST,
}

impl TryFrom<&str> for HttpMethod {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            other => Err(format!("invalid http method: {}", other)),
        }
    }
}

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

pub struct Request<'a> {
    pub request_line: RequestLine<'a>,
}

impl<'a> Request<'a> {
    pub fn parse(buf: &'a str) -> Result<Request<'a>, String> {
        let lines: Vec<_> = buf.split("\r\n").collect();

        let request_line = RequestLine::parse(lines[0])?;

        Ok(Self { request_line })
    }
}
