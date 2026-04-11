use std::net::{Shutdown, TcpStream};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum NetworkOperation {
    Ping(String, usize),
    Curl(String),
    Scan(String, u16, u16),
}

impl NetworkOperation {
    pub fn execute(&self) -> Result<String, String> {
        match self {
            NetworkOperation::Ping(host, count) => Self::ping(host, *count),
            NetworkOperation::Curl(url) => Self::curl(url),
            NetworkOperation::Scan(host, start_port, end_port) => {
                Self::scan_ports(host, *start_port, *end_port)
            }
        }
    }

    fn ping(host: &str, count: usize) -> Result<String, String> {
        let count = count.max(1).min(10);

        let output = Command::new("ping")
            .args(["-n", &count.to_string(), host])
            .output()
            .map_err(|e| format!("Failed to run ping: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn curl(url: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(url)
            .send()
            .map_err(|e| format!("Failed to fetch URL: {}", e))?;

        if response.status().is_success() {
            response
                .text()
                .map_err(|e| format!("Failed to read response: {}", e))
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }

    fn scan_ports(host: &str, start_port: u16, end_port: u16) -> Result<String, String> {
        let mut open_ports = Vec::new();

        for port in start_port..=end_port {
            let addr = format!("{}:{}", host, port);
            if let Ok(_) = TcpStream::connect_timeout(
                &addr
                    .parse()
                    .map_err(|e| format!("Invalid address: {}", e))?,
                Duration::from_millis(200),
            ) {
                open_ports.push(port);
            }
        }

        if open_ports.is_empty() {
            Ok(format!(
                "No open ports found on {} (scanned {}-{})",
                host, start_port, end_port
            ))
        } else {
            Ok(format!("Open ports on {}: {:?}", host, open_ports))
        }
    }
}

pub fn parse_network_command(args: &[&str]) -> Result<NetworkOperation, String> {
    if args.is_empty() {
        return Err(
            "Usage: ping <host> [count] | curl <url> | scan <host> [start] [end]".to_string(),
        );
    }

    match args[0] {
        "ping" => {
            if args.len() < 2 {
                return Err("Usage: ping <host> [count]".to_string());
            }
            let count: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
            Ok(NetworkOperation::Ping(args[1].to_string(), count))
        }
        "curl" => {
            if args.len() < 2 {
                return Err("Usage: curl <url>".to_string());
            }
            Ok(NetworkOperation::Curl(args[1].to_string()))
        }
        "scan" => {
            if args.len() < 2 {
                return Err("Usage: scan <host> [start_port] [end_port]".to_string());
            }
            let start: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
            let end: u16 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1024);
            Ok(NetworkOperation::Scan(args[1].to_string(), start, end))
        }
        _ => Err(format!("Unknown network command: {}", args[0])),
    }
}
