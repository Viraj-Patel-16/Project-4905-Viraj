use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetProtocol {
    Http,
    Tcp,
    Udp,
}

impl TargetProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "HTTP",
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Http => Self::Tcp,
            Self::Tcp => Self::Udp,
            Self::Udp => Self::Http,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Http => Self::Udp,
            Self::Tcp => Self::Http,
            Self::Udp => Self::Tcp,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetSystem {
    Generic,
    Nginx,
    HAProxy,
}

impl TargetSystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "Generic",
            Self::Nginx => "NGINX",
            Self::HAProxy => "HAProxy",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Generic => Self::Nginx,
            Self::Nginx => Self::HAProxy,
            Self::HAProxy => Self::Generic,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Generic => Self::HAProxy,
            Self::Nginx => Self::Generic,
            Self::HAProxy => Self::Nginx,
        }
    }

    pub fn default_endpoint(self, protocol: TargetProtocol) -> &'static str {
        match protocol {
            TargetProtocol::Http => "http://127.0.0.1:8080",
            TargetProtocol::Tcp => "127.0.0.1:9000",
            TargetProtocol::Udp => "127.0.0.1:9001",
        }
    }

    pub fn default_http_path(self) -> &'static str {
        "/traffic"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub enabled: bool,
    pub system: TargetSystem,
    pub protocol: TargetProtocol,
    pub endpoint: String,
    pub http_path: String,
}

impl Default for TargetConfig {
    fn default() -> Self {
        let system = TargetSystem::Generic;
        let protocol = TargetProtocol::Http;
        Self {
            enabled: false,
            system,
            protocol,
            endpoint: system.default_endpoint(protocol).to_string(),
            http_path: system.default_http_path().to_string(),
        }
    }
}
