use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

use crate::models::KnockStep;

#[derive(Debug, Serialize, Deserialize)]
pub struct KnockResult {
    pub success: bool,
    pub message: String,
}

pub fn perform_knock(host: &str, ports_json: &str, protocol: &str, delay_ms: u64) -> KnockResult {
    let steps: Vec<KnockStep> = match serde_json::from_str(ports_json) {
        Ok(s) => s,
        Err(e) => {
            return KnockResult {
                success: false,
                message: format!("Invalid knock ports JSON: {}", e),
            }
        }
    };

    if steps.is_empty() {
        return KnockResult {
            success: false,
            message: "No knock ports configured".into(),
        };
    }

    let delay = Duration::from_millis(delay_ms.max(10));
    let total_steps = steps.len();

    for (i, step) in steps.iter().enumerate() {
        let proto = if step.protocol.is_empty() {
            protocol
        } else {
            &step.protocol
        };

        let addr: SocketAddr = match format!("{}:{}", host, step.port).parse() {
            Ok(a) => a,
            Err(e) => {
                return KnockResult {
                    success: false,
                    message: format!("Invalid address {}:{}: {}", host, step.port, e),
                }
            }
        };

        match proto.to_lowercase().as_str() {
            "udp" => {
                if let Err(e) = send_udp_knock(&addr) {
                    log::warn!("UDP knock to {} failed (non-fatal): {}", addr, e);
                }
            }
            "tcp" => {
                if let Err(e) = send_tcp_knock(&addr) {
                    log::warn!("TCP knock to {} failed (non-fatal): {}", addr, e);
                }
            }
            other => {
                return KnockResult {
                    success: false,
                    message: format!("Unknown protocol: {}", other),
                };
            }
        }

        if i < total_steps - 1 {
            std::thread::sleep(delay);
        }
    }

    KnockResult {
        success: true,
        message: format!(
            "Knock sequence complete: {} packets sent to {}",
            total_steps, host
        ),
    }
}

fn send_udp_knock(addr: &SocketAddr) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;
    socket.send_to(&[0], addr)?;
    Ok(())
}

fn send_tcp_knock(addr: &SocketAddr) -> std::io::Result<()> {
    let _stream = TcpStream::connect_timeout(addr, Duration::from_secs(2))?;
    Ok(())
}
