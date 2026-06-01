use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub async fn http_ok(client: &reqwest::Client, url: &str) -> bool {
    client
        .get(url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

pub async fn get_json(client: &reqwest::Client, url: &str) -> Option<serde_json::Value> {
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().await.ok()
}

pub fn tcp_ok(addr: &str) -> bool {
    let Ok(mut resolved) = addr.to_socket_addrs() else {
        return false;
    };
    match resolved.next() {
        Some(socket) => TcpStream::connect_timeout(&socket, Duration::from_secs(2)).is_ok(),
        None => false,
    }
}
