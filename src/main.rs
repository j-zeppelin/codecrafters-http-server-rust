use std::io::{BufRead, BufReader, Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

use crate::request::Request;

mod request;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);
                let mut request = String::new();

                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();

                    if line == "\r\n" {
                        break;
                    }
                    request.push_str(&line);
                }

                let request = Request::parse(&request).unwrap();

                let return_msg = match request.request_line.target {
                    "/" => "HTTP/1.1 200 OK\r\n\r\n",
                    _ => "HTTP/1.1 404 Not Found\r\n\r\n",
                };

                stream.write_all(return_msg.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    const BASE_URL: &str = "http://127.0.0.1:4221";

    #[test]
    fn root_endpoint_returns_200() -> anyhow::Result<()> {
        let response = reqwest::blocking::get(format!("{BASE_URL}"))?;

        assert!(response.status().as_u16() == 200);
        Ok(())
    }

    #[test]
    fn unknown_path_returns_404() -> anyhow::Result<()> {
        let response = reqwest::blocking::get(format!("{BASE_URL}/amogus"))?;

        assert!(response.status().as_u16() == 404);
        Ok(())
    }
}
