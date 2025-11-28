use std::net::Ipv4Addr;
use std::str::FromStr;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
    InvalidIp,
    InvalidRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid format. Expected format: 192.168.1.1-254"),
            ParseError::InvalidIp => write!(f, "Invalid IP address"),
            ParseError::InvalidRange => write!(f, "Invalid range. Start must be less than or equal to end, and within 0-255"),
        }
    }
}

pub fn parse_ip_range(range_str: &str) -> Result<Vec<Ipv4Addr>, ParseError> {
    // Expected format: "10.10.0.1-254"
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err(ParseError::InvalidFormat);
    }

    let start_ip_str = parts[0];
    let end_suffix = parts[1];

    let start_ip = Ipv4Addr::from_str(start_ip_str).map_err(|_| ParseError::InvalidIp)?;
    let octets = start_ip.octets();
    
    let end_octet: u8 = end_suffix.parse().map_err(|_| ParseError::InvalidRange)?;
    let start_octet = octets[3];

    if start_octet > end_octet {
        return Err(ParseError::InvalidRange);
    }

    let mut ips = Vec::new();
    for i in start_octet..=end_octet {
        ips.push(Ipv4Addr::new(octets[0], octets[1], octets[2], i));
    }

    Ok(ips)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_range() {
        let ips = parse_ip_range("192.168.1.1-5").unwrap();
        assert_eq!(ips.len(), 5);
        assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(ips[4], Ipv4Addr::new(192, 168, 1, 5));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(parse_ip_range("192.168.1.1").is_err());
    }
}
