use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::HeaderMap;
use axum::http::request::Parts;
use clinicore_core::config::ClientIpSource;

use crate::AppState;

const CF_CONNECTING_IP: &str = "cf-connecting-ip";
const X_FORWARDED_FOR: &str = "x-forwarded-for";
const UNKNOWN: &str = "unknown";
const IPV6_NETWORK_SEGMENTS: usize = 4;
const IPV6_RANGE_SEGMENTS: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientIp {
    pub address: Option<IpAddr>,
}

impl ClientIp {
    pub fn key(&self) -> String {
        match self.address {
            None => UNKNOWN.to_owned(),
            Some(IpAddr::V4(address)) => address.to_string(),
            Some(IpAddr::V6(address)) => {
                let mut segments = address.segments();
                segments[IPV6_NETWORK_SEGMENTS..].fill(0);
                format!("{}/64", Ipv6Addr::from(segments))
            }
        }
    }

    pub fn network_key(&self) -> String {
        match self.address {
            None => UNKNOWN.to_owned(),
            Some(IpAddr::V4(address)) => {
                let [first, second, third, _] = address.octets();
                format!("{}/24", Ipv4Addr::new(first, second, third, 0))
            }
            Some(IpAddr::V6(address)) => {
                let mut segments = address.segments();
                segments[IPV6_RANGE_SEGMENTS..].fill(0);
                format!("{}/48", Ipv6Addr::from(segments))
            }
        }
    }
}

impl FromRequestParts<AppState> for ClientIp {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(address)| address.ip());
        let address = read(state.config.client_ip_source, &parts.headers, peer)
            .map(|address| address.to_canonical());
        if address.is_none() {
            tracing::warn!("client ip unavailable");
        }
        Ok(Self { address })
    }
}

fn read(source: ClientIpSource, headers: &HeaderMap, peer: Option<IpAddr>) -> Option<IpAddr> {
    match source {
        ClientIpSource::Peer => peer,
        ClientIpSource::CfConnectingIp => headers
            .get(CF_CONNECTING_IP)?
            .to_str()
            .ok()?
            .trim()
            .parse()
            .ok(),
        ClientIpSource::XForwardedFor(hops) => {
            let entries = headers
                .get_all(X_FORWARDED_FOR)
                .iter()
                .map(|value| value.to_str().ok())
                .collect::<Option<Vec<_>>>()?;
            entries
                .iter()
                .flat_map(|value| value.split(','))
                .rev()
                .nth(hops - 1)?
                .trim()
                .parse()
                .ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network_key(address: Option<&str>) -> String {
        ClientIp {
            address: address.map(|address| address.parse().expect("the address parses")),
        }
        .network_key()
    }

    #[test]
    fn the_network_is_the_24_of_an_ipv4_and_the_48_of_an_ipv6() {
        assert_eq!(network_key(Some("203.0.113.77")), "203.0.113.0/24");
        assert_eq!(
            network_key(Some("2001:db8:aaaa:bbbb::1")),
            "2001:db8:aaaa::/48"
        );
        assert_eq!(network_key(None), "unknown");
    }
}
