use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    time::Instant,
};

use crate::server::{http::HttpStatus, response::Response};

use self::request::Request;

mod http;
mod request;
mod response;

const SERVER_ADDR: &str = "127.0.0.1:4221";

pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// panics if the `TcpListener` can not be bound to `SERVER_ADDR`
    pub fn new() -> Self {
        let listener = TcpListener::bind(SERVER_ADDR).unwrap_or_else(|e| {
            panic!("Failed to bind to {SERVER_ADDR}: {e}");
        });

        Self { listener }
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let start = Instant::now();

                    let request = match Self::read_request(&stream) {
                        Ok(req) => req,
                        Err(e) => {
                            Self::send_error(&mut stream, HttpStatus::InternalServerError, &e);
                            eprintln!("failed to read request: {e}"));
                        };
                    };

                    // actually try to parse the request
                    let request = match Request::parse(&request) {
                        Ok(req) => req,
                        Err(e) => {
                            Self::send_error(&mut stream, HttpStatus::BadRequest, &e);
                            eprintln!("{e}");
                        }
                    };

                    Self::handle_request(&request, stream, start);

                    if let Err(err) = self.handle_stream(stream, start) {
                        eprintln!("{err}");
                    }
                }
                Err(e) => {
                    println!("error: {e}");
                }
            }
        }
    }

    fn handle_stream(&self, mut stream: TcpStream, start: Instant) -> Result<(), String> {
        // read request from stream
        let request = match Self::read_request(&stream) {
            Ok(req) => req,
            Err(e) => {
                Self::send_error(&mut stream, HttpStatus::InternalServerError, &e);
                return Err(format!("failed to read request: {e}"));
            }
        };

        // actually try to parse the request
        let request = match Request::parse(&request) {
            Ok(req) => req,
            Err(e) => {
                Self::send_error(&mut stream, HttpStatus::BadRequest, &e);
                return Err(e);
            }
        };

        Self::handle_request(&request, stream, start);
        Ok(())
    }

    fn read_request(stream: &TcpStream) -> Result<String, String> {
        let mut reader = BufReader::new(stream);
        let mut request = String::new();

        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("could not read line from request: {e}"))?;

            if line == "\r\n" {
                break;
            }
            request.push_str(&line);
        }

        Ok(request)
    }

    fn handle_request(request: &Request, mut stream: TcpStream, start: Instant) {
        let segments = request
            .request_line
            .target
            .trim_start_matches('/')
            .split('/')
            .collect::<Vec<_>>();

        let response = match segments.as_slice() {
            [""] => Response::builder().status(HttpStatus::Ok).build(),
            ["echo", msg] => Self::handle_echo(msg),
            _ => Response::builder().status(HttpStatus::NotFound).build(),
        };

        if let Err(err) = stream.write_all(response.to_string().as_bytes()) {
            eprintln!("could not write response: {err}");
        }

        Self::log_request(request, start);
    }

    fn handle_echo(msg: &str) -> Response {
        Response::builder()
            .status(HttpStatus::Ok)
            .headers(vec![
                ("Content-Type", "text/plain"),
                ("Content-Length", &msg.len().to_string()),
            ])
            .body(msg)
            .build()
    }

    fn log_request(req: &Request, start: Instant) {
        println!(
            "[{}] {} {:.3}s",
            req.timestamp.format("%Y-%m-%d %H:%M:%S"),
            req.request_line,
            start.elapsed().as_secs_f64()
        );
    }

    fn send_error(stream: &mut TcpStream, status: HttpStatus, message: impl Into<String>) {
        let response = Response::builder().status(status).body(message).build();
        if let Err(e) = stream.write_all(response.to_string().as_bytes()) {
            eprintln!("could not write error response: {e}");
        }
    }
}
