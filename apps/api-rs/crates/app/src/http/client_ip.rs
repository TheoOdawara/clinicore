use std::net::IpAddr;

use axum_client_ip::{ClientIp, Rejection};
use ipnet::Ipv6Net;

pub fn tracker(client: Result<ClientIp, Rejection>) -> String {
    let Ok(ClientIp(address)) = client else {
        return "unknown".to_string();
    };
    match address.to_canonical() {
        IpAddr::V6(address) => Ipv6Net::new_assert(address, 64).trunc().to_string(),
        address => address.to_string(),
    }
}
