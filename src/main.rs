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
}
