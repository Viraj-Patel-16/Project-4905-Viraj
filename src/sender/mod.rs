use std::io::Write;
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Socket, Type};

use crate::model::{TargetConfig, TargetProtocol, TrafficEvent};

#[derive(Debug, Clone, Default)]
pub struct SendReport {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub last_error: Option<String>,
}

pub fn send_events(events: &[TrafficEvent], config: &TargetConfig) -> SendReport {
    let mut report = SendReport::default();

    if !config.enabled || events.is_empty() {
        return report;
    }

    match config.protocol {
        TargetProtocol::Http => send_http(events, config, &mut report),
        TargetProtocol::Tcp => send_tcp(events, config, &mut report),
        TargetProtocol::Udp => send_udp(events, config, &mut report),
    }

    report
}

fn build_http_request(event: &TrafficEvent, config: &TargetConfig) -> Result<Vec<u8>, String> {
    let body = serde_json::to_vec(event).map_err(|error| format!("JSON encode error: {error}"))?;
    let path = config.http_path.trim_start_matches('/');
    let host = parse_http_host(&config.endpoint)?;

    let request = format!(
        "POST /{path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );

    let mut bytes = request.into_bytes();
    bytes.extend_from_slice(&body);
    Ok(bytes)
}

fn connect_with_timeout(addr: &str, timeout_secs: u64) -> Result<TcpStream, String> {
    let addr: SocketAddr = addr
        .parse()
        .map_err(|e| format!("Failed to parse address: {}", e))?;

    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)
        .map_err(|e| format!("Socket creation failed: {}", e))?;

    socket
        .connect_timeout(&addr.into(), Duration::from_secs(timeout_secs))
        .map_err(|e| format!("Connection failed or timed out: {}", e))?;

    Ok(TcpStream::from(socket))
}

fn strip_scheme_and_path(endpoint: &str) -> String {
    let trimmed = endpoint.trim();
    let without_scheme = trimmed
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("tcp://")
        .trim_start_matches("udp://");
    without_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn parse_http_host(endpoint: &str) -> Result<String, String> {
    let host = strip_scheme_and_path(endpoint);
    if host.is_empty() {
        return Err("HTTP endpoint is empty".to_string());
    }
    Ok(host)
}

fn parse_socket_target(endpoint: &str, protocol: TargetProtocol) -> Result<String, String> {
    let addr = strip_scheme_and_path(endpoint);
    if addr.is_empty() {
        return Err(format!("{} endpoint is empty", protocol.as_str()));
    }
    Ok(addr)
}

fn send_http(events: &[TrafficEvent], config: &TargetConfig, report: &mut SendReport) {
    let host = match parse_http_host(&config.endpoint) {
        Ok(host) => host,
        Err(error) => {
            report.failed = events.len();
            report.attempted = events.len();
            report.last_error = Some(error);
            return;
        }
    };

    for event in events {
        report.attempted += 1;
        match build_http_request(event, config) {
            Ok(bytes) => match connect_with_timeout(&host, 3) {
                Ok(mut stream) => {
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                    match stream.write_all(&bytes) {
                        Ok(()) => report.succeeded += 1,
                        Err(error) => {
                            report.failed += 1;
                            report.last_error = Some(format!(
                                "HTTP write error for request_id {}: {}",
                                event.request_id, error
                            ));
                        }
                    }
                }
                Err(error) => {
                    report.failed += 1;
                    report.last_error = Some(format!(
                        "HTTP connect error for request_id {}: {}",
                        event.request_id, error
                    ));
                }
            },
            Err(error) => {
                report.failed += 1;
                report.last_error = Some(format!(
                    "HTTP request build failed for request_id {}: {}",
                    event.request_id, error
                ));
            }
        }
    }
}

fn send_tcp(events: &[TrafficEvent], config: &TargetConfig, report: &mut SendReport) {
    let endpoint = match parse_socket_target(&config.endpoint, TargetProtocol::Tcp) {
        Ok(endpoint) => endpoint,
        Err(error) => {
            report.failed = events.len();
            report.attempted = events.len();
            report.last_error = Some(error);
            return;
        }
    };

    let mut stream = match connect_with_timeout(&endpoint, 3) {
        Ok(stream) => stream,
        Err(error) => {
            report.failed = events.len();
            report.attempted = events.len();
            report.last_error = Some(format!("TCP connect failed: {}", error));
            return;
        }
    };

    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));

    for event in events {
        report.attempted += 1;
        match serde_json::to_vec(event) {
            Ok(mut bytes) => {
                bytes.push(b'\n');
                match stream.write_all(&bytes) {
                    Ok(()) => report.succeeded += 1,
                    Err(error) => {
                        report.failed += 1;
                        report.last_error = Some(format!(
                            "TCP write error for request_id {}: {}",
                            event.request_id, error
                        ));
                    }
                }
            }
            Err(error) => {
                report.failed += 1;
                report.last_error = Some(format!(
                    "JSON encode error for request_id {}: {}",
                    event.request_id, error
                ));
            }
        }
    }
}

fn send_udp(events: &[TrafficEvent], config: &TargetConfig, report: &mut SendReport) {
    let endpoint = match parse_socket_target(&config.endpoint, TargetProtocol::Udp) {
        Ok(endpoint) => endpoint,
        Err(error) => {
            report.failed = events.len();
            report.attempted = events.len();
            report.last_error = Some(error);
            return;
        }
    };

    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(error) => {
            report.failed = events.len();
            report.attempted = events.len();
            report.last_error = Some(format!("UDP bind failed: {}", error));
            return;
        }
    };

    let _ = socket.set_write_timeout(Some(Duration::from_secs(3)));

    for event in events {
        report.attempted += 1;
        match serde_json::to_vec(event) {
            Ok(bytes) => match socket.send_to(&bytes, &endpoint) {
                Ok(_) => report.succeeded += 1,
                Err(error) => {
                    report.failed += 1;
                    report.last_error = Some(format!(
                        "UDP send error for request_id {}: {}",
                        event.request_id, error
                    ));
                }
            },
            Err(error) => {
                report.failed += 1;
                report.last_error = Some(format!(
                    "JSON encode error for request_id {}: {}",
                    event.request_id, error
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_http_request_uses_configured_host_and_path() {
        let event = TrafficEvent::new(1_700_000_000, "tenant-1", 42, 128, "NGINX");
        let config = TargetConfig {
            enabled: true,
            system: crate::model::TargetSystem::Nginx,
            protocol: TargetProtocol::Http,
            endpoint: "http://127.0.0.1:8080".to_string(),
            http_path: "/traffic".to_string(),
        };

        let request = build_http_request(&event, &config).expect("request should build");
        let request_text = String::from_utf8(request).expect("request should be UTF-8");

        assert!(request_text.starts_with("POST /traffic HTTP/1.1\r\n"));
        assert!(request_text.contains("Host: 127.0.0.1:8080\r\n"));
        assert!(request_text.contains("Content-Type: application/json\r\n"));
        assert!(request_text.contains("Content-Length:"));
    }

    #[test]
    fn socket_target_parsing_accepts_scheme_and_path() {
        let tcp = parse_socket_target("tcp://127.0.0.1:9000/ingest", TargetProtocol::Tcp)
            .expect("tcp endpoint should parse");
        let udp = parse_socket_target("http://127.0.0.1:9001/traffic", TargetProtocol::Udp)
            .expect("udp endpoint should parse");

        assert_eq!(tcp, "127.0.0.1:9000");
        assert_eq!(udp, "127.0.0.1:9001");
    }
}
