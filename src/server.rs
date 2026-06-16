use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};

use anyhow::Result;

use crate::server::{
    http::{HttpMethod, HttpStatus},
    response::Response,
};

use self::request::Request;

mod http;
mod request;
mod response;

const SERVER_ADDR: &str = "127.0.0.1:4221";

pub struct Server {
    listener: TcpListener,
    root_dir: PathBuf,
}

impl Server {
    /// panics if the `TcpListener` can not be bound to `SERVER_ADDR`
    pub fn new(root_dir: PathBuf) -> Self {
        let listener = TcpListener::bind(SERVER_ADDR).unwrap_or_else(|e| {
            panic!("Failed to bind to {SERVER_ADDR}: {e}");
        });

        Self { listener, root_dir }
    }

    pub fn run(self: Arc<Self>) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server = Arc::clone(&self);
                    std::thread::spawn(move || {
                        if let Err(err) = server.handle_connection(stream) {
                            println!("{err}");
                        }
                    });
                }
                Err(e) => {
                    println!("error: {e}");
                }
            }
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) -> Result<(), String> {
        let start = Instant::now();

        // try to read the request from the stream
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
                return Err(format!("{e}"));
            }
        };

        self.handle_request(&request, stream, start);
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

    fn handle_request(&self, request: &Request, mut stream: TcpStream, start: Instant) {
        let method = &request.line.method;
        let segments = request
            .line
            .target
            .trim_start_matches('/')
            .split('/')
            .collect::<Vec<_>>();

        let response = match segments.as_slice() {
            [""] => Response::builder().status(HttpStatus::Ok).build(),
            ["echo", msg] if matches!(method, HttpMethod::Get) => Self::handle_echo(msg),
            ["user-agent"] if matches!(method, HttpMethod::Get) => {
                Self::handle_user_agent(&request)
            }
            ["files", file_name] if matches!(method, HttpMethod::Get) => {
                self.handle_files(file_name)
            }
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

    fn handle_user_agent(req: &Request) -> Response {
        let Some(user_agent) = req.headers.get("user-agent") else {
            return Response::builder()
                .status(HttpStatus::BadRequest)
                .headers(vec![
                    ("Content-Type", "text/plain"),
                    ("Content-Length", "27"),
                ])
                .body("User-Agent header not found")
                .build();
        };

        Response::builder()
            .status(HttpStatus::Ok)
            .headers(vec![
                ("Content-Type", "text/plain"),
                ("Content-Length", &user_agent.len().to_string()),
            ])
            .body(*user_agent)
            .build()
    }

    fn handle_files(&self, file_name: &str) -> Response {
        let file_path = self.root_dir.join(file_name);
        let Ok(contents) = fs::read_to_string(file_path) else {
            return Response::builder().status(HttpStatus::NotFound).build();
        };

        Response::builder()
            .status(HttpStatus::Ok)
            .headers(vec![
                ("Content-Type", "application/octet-stream"),
                ("Content-Length", &contents.len().to_string()),
            ])
            .body(contents)
            .build()
    }

    fn log_request(req: &Request, start: Instant) {
        println!(
            "[{}] {} {:.3}s",
            req.timestamp.format("%Y-%m-%d %H:%M:%S"),
            req.line,
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
