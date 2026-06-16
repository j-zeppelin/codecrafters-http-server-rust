#[allow(unused_imports)]
use std::net::TcpListener;

use crate::server::Server;

mod server;

fn main() {
    let server = Server::new();
    server.run();
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

    #[test]
    fn echo_endpoint_returns_correct_body_and_headers() -> anyhow::Result<()> {
        let response = reqwest::blocking::get(format!("{BASE_URL}/echo/abc"))?;

        assert!(response.status() == 200);

        assert_eq!(
            response.headers().get("Content-Type").unwrap().to_str()?,
            "text/plain"
        );

        assert_eq!(
            response.headers().get("Content-Length").unwrap().to_str()?,
            "3"
        );

        assert_eq!(response.text()?, "abc");
        Ok(())
    }

    #[test]
    fn user_agent_endpoint_returns_correct_body_and_headers() -> anyhow::Result<()> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(format!("{BASE_URL}/user-agent"))
            .header("User-Agent", "test-user-agent")
            .send()?;

        assert_eq!(
            response.headers().get("Content-Type").unwrap().to_str()?,
            "text/plain"
        );
        assert_eq!(
            response.headers().get("Content-Length").unwrap().to_str()?,
            "15"
        );
        assert_eq!(response.text()?, "test-user-agent");

        Ok(())
    }
}
