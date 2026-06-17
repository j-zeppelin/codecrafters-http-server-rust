use std::{
    fs,
    io::{BufReader, Write},
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

        println!("root directory set to {}", root_dir.display());
        println!("server running on {SERVER_ADDR}");
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

        // try to parse the request
        let request = match Request::parse(&mut BufReader::new(&stream)) {
            Ok(req) => req,
            Err(e) => {
                Self::send_error(&mut stream, HttpStatus::BadRequest, &e);
                return Err(format!("{e}"));
            }
        };

        self.handle_request(request, stream, start);
        Ok(())
    }

    fn handle_request(&self, request: Request, mut stream: TcpStream, start: Instant) {
        let method = &request.line.method;
        let segments = &request
            .line
            .target
            .trim_start_matches('/')
            .split('/')
            .collect::<Vec<_>>();

        Self::log_request(&request, start);

        let response = match segments.as_slice() {
            [""] => Response::builder().status(HttpStatus::Ok).build(),
            ["echo", msg] if matches!(method, HttpMethod::Get) => Self::handle_echo(msg),
            ["user-agent"] if matches!(method, HttpMethod::Get) => {
                Self::handle_user_agent(&request)
            }
            ["files", file_name] if matches!(method, HttpMethod::Get) => {
                self.handle_file_get(file_name)
            }
            ["files", file_name] if matches!(method, HttpMethod::Post) => {
                self.handle_file_post(file_name, &request)
            }
            _ => Response::builder().status(HttpStatus::NotFound).build(),
        };

        let response: Response =
            if let Some(accept_encoding) = request.headers.get("accept-encoding") {
                if accept_encoding.contains("gzip") {
                    let body = response.body.unwrap();
                    Response::builder()
                        .status(response.status)
                        .headers(vec![
                            ("Content-Type", "application/octet-stream"),
                            ("Content-Length", &body.len().to_string()),
                            ("Content-Encoding", "gzip"),
                        ])
                        .body(body)
                        .build()
                } else {
                    response
                }
            } else {
                response
            };

        if let Err(err) = stream.write_all(response.to_string().as_bytes()) {
            eprintln!("could not write response: {err}");
        }
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
            .body(user_agent)
            .build()
    }

    fn handle_file_get(&self, file_name: &str) -> Response {
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

    fn handle_file_post(&self, file_name: &str, request: &Request) -> Response {
        let file_path = self.root_dir.join(file_name);

        let Some(body) = request.body.as_ref() else {
            return Response::builder()
                .status(HttpStatus::BadRequest)
                .body("empty body")
                .build();
        };

        if let Err(err) = fs::write(file_path, body) {
            return Response::builder()
                .status(HttpStatus::InternalServerError)
                .body(format!("could not write to file: {err}"))
                .build();
        }

        Response::builder().status(HttpStatus::Created).build()
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
