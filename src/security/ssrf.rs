use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

static BLOCKED_HOSTS: &[&str] = &[
    "169.254.169.254",
    "metadata.google.internal",
    "metadata.google.com",
    "100.100.100.200",
];

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || *v4 == Ipv4Addr::UNSPECIFIED
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_private_ip(&IpAddr::V4(v4));
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

pub fn check_ssrf(url_str: &str) -> Option<String> {
    let parsed = url::Url::parse(url_str).ok()?;
    let host = parsed.host()?;

    let host_str = match &host {
        url::Host::Domain(d) => d.to_string(),
        url::Host::Ipv4(v4) => v4.to_string(),
        url::Host::Ipv6(v6) => v6.to_string(),
    };

    if BLOCKED_HOSTS.contains(&host_str.as_str()) {
        return Some(format!("Blocked host: {} (cloud metadata endpoint)", host_str));
    }

    match host {
        url::Host::Ipv4(v4) => {
            let ip = IpAddr::V4(v4);
            if is_private_ip(&ip) {
                return Some(format!("URL resolves to private IP: {}", ip));
            }
        }
        url::Host::Ipv6(v6) => {
            let ip = if let Some(v4) = v6.to_ipv4_mapped() {
                IpAddr::V4(v4)
            } else {
                IpAddr::V6(v6)
            };
            if is_private_ip(&ip) {
                return Some(format!("URL resolves to private IP: {}", ip));
            }
        }
        url::Host::Domain(_) => {
            let port = parsed.port().unwrap_or(match parsed.scheme() {
                "https" => 443,
                _ => 80,
            });
            let addr_str = format!("{}:{}", host_str, port);
            if let Ok(addrs) = addr_str.to_socket_addrs() {
                for addr in addrs {
                    if is_private_ip(&addr.ip()) {
                        return Some(format!("URL resolves to private IP: {}", addr.ip()));
                    }
                }
            }
        }
    }

    None
}
