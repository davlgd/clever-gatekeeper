use crate::config;

/// IP validation and blocking module

/// Check if an IP address is in the blocked list
pub fn is_ip_blocked(ip: &str) -> bool {
    let blocked_ips = config::get_blocked_ips();
    blocked_ips.iter().any(|blocked_ip| blocked_ip == ip)
}

/// Extract real client IP from load balancer headers
///
/// This function implements the security model:
/// 1. Requires both x-forwarded-for and forwarded headers
/// 2. Validates that the proxy IP (from 'by=' field) is in allowlist
/// 3. Extracts the real client IP (last valid IP in x-forwarded-for chain)
pub fn extract_and_validate_real_ip(headers: &hyper::HeaderMap) -> Option<String> {
    // 1. x-forwarded-for header is mandatory
    let xff = headers.get("x-forwarded-for")?.to_str().ok()?;

    // 2. forwarded header is mandatory for proxy validation
    let forwarded = headers.get("forwarded")?.to_str().ok()?;

    // 3. Extract proxy IP from 'by=' field in forwarded header
    let proxy_ip = extract_proxy_ip_from_forwarded(forwarded)?;

    // 4. Validate proxy IP is in allowlist
    if !is_proxy_ip_allowed(&proxy_ip) {
        return None;
    }

    // 5. Extract real client IP (last valid IP in forwarded chain)
    extract_client_ip_from_xff(xff)
}

/// Extract proxy IP from forwarded header 'by=' field
fn extract_proxy_ip_from_forwarded(forwarded: &str) -> Option<String> {
    forwarded
        .split(';')
        .find_map(|part| part.trim().strip_prefix("by="))
        .map(|ip| ip.trim().to_string())
}

/// Check if proxy IP is in the allowed list
fn is_proxy_ip_allowed(proxy_ip: &str) -> bool {
    config::get_allowed_proxy_ips()
        .map(|allowed_ips| allowed_ips.iter().any(|ip| ip == proxy_ip))
        .unwrap_or(false)
}

/// Extract client IP from x-forwarded-for header (last valid IP)
fn extract_client_ip_from_xff(xff: &str) -> Option<String> {
    xff.split(',')
        .map(|ip| ip.trim())
        .filter(|ip| is_valid_ip_format(ip))
        .last()
        .map(|ip| ip.to_string())
}

/// Basic IP format validation (contains . for IPv4 or : for IPv6)
fn is_valid_ip_format(ip: &str) -> bool {
    ip.contains('.') || ip.contains(':')
}
