use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{HeaderValue, Method, Request};
use axum::routing::get;
use clinicore_app::http::client_ip::ClientIp;
use clinicore_app::http::layers;
use clinicore_core::config::AppEnv;

use crate::support::{Captured, dispatch, state_with};

async fn key_of(source: &str, peer: Option<&str>, headers: &[(&str, &str)]) -> String {
    let headers: Vec<(&str, &[u8])> = headers
        .iter()
        .map(|(name, value)| (*name, value.as_bytes()))
        .collect();
    key_of_bytes(source, peer, &headers).await
}

async fn key_of_bytes(source: &str, peer: Option<&str>, headers: &[(&str, &[u8])]) -> String {
    let app = layers(
        Router::new().route("/ip", get(|ip: ClientIp| async move { ip.key() })),
        state_with(AppEnv::Test, &[("CLIENT_IP_SOURCE", source)]),
    );
    let mut request = Request::builder().method(Method::GET).uri("/ip");
    for (name, value) in headers {
        request = request.header(
            *name,
            HeaderValue::from_bytes(value).expect("the test header is a valid value"),
        );
    }
    if let Some(peer) = peer {
        let peer: SocketAddr = peer.parse().expect("the peer is a socket address");
        request = request.extension(ConnectInfo(peer));
    }
    let request = request
        .body(Body::empty())
        .expect("the request is well formed");
    String::from_utf8(dispatch(app, request).await.body).expect("the key is UTF-8")
}

#[tokio::test]
async fn the_peer_is_read_with_mapped_ipv4_unwrapped_and_ipv6_by_its_64() {
    assert_eq!(
        key_of("peer", Some("203.0.113.7:51000"), &[]).await,
        "203.0.113.7"
    );
    assert_eq!(
        key_of("peer", Some("[::ffff:203.0.113.7]:51000"), &[]).await,
        "203.0.113.7"
    );
    assert_eq!(
        key_of("peer", Some("[2001:db8:1:2:3:4:5:6]:51000"), &[]).await,
        "2001:db8:1:2::/64"
    );
    assert_eq!(
        key_of("peer", None, &[("x-forwarded-for", "198.51.100.1")]).await,
        "unknown"
    );
}

#[tokio::test]
async fn cf_connecting_ip_is_read_and_a_missing_or_bad_value_counts_as_unknown() {
    let peer = Some("10.0.0.2:51000");
    assert_eq!(
        key_of(
            "cf-connecting-ip",
            peer,
            &[("cf-connecting-ip", " 198.51.100.9 ")]
        )
        .await,
        "198.51.100.9"
    );
    assert_eq!(key_of("cf-connecting-ip", peer, &[]).await, "unknown");
    assert_eq!(
        key_of(
            "cf-connecting-ip",
            peer,
            &[("cf-connecting-ip", "not-an-ip")]
        )
        .await,
        "unknown"
    );
}

#[tokio::test]
async fn x_forwarded_for_reads_the_nth_entry_from_the_right_and_ignores_the_forged_left() {
    let peer = Some("10.0.0.2:51000");
    assert_eq!(
        key_of(
            "x-forwarded-for:2",
            peer,
            &[("x-forwarded-for", "1.1.1.1, 198.51.100.9, 10.0.0.1")]
        )
        .await,
        "198.51.100.9"
    );
    assert_eq!(
        key_of(
            "x-forwarded-for:2",
            peer,
            &[
                ("x-forwarded-for", "1.1.1.1, 198.51.100.9"),
                ("x-forwarded-for", "10.0.0.1"),
            ]
        )
        .await,
        "198.51.100.9"
    );
    assert_eq!(
        key_of(
            "x-forwarded-for:2",
            peer,
            &[("x-forwarded-for", "10.0.0.1")]
        )
        .await,
        "unknown"
    );
    assert_eq!(key_of("x-forwarded-for:1", peer, &[]).await, "unknown");
    assert_eq!(
        key_of("x-forwarded-for:1", peer, &[("x-forwarded-for", "garbage")]).await,
        "unknown"
    );
}

#[tokio::test]
async fn a_header_that_is_not_text_counts_as_unknown() {
    let peer = Some("10.0.0.2:51000");
    for (source, header) in [
        ("cf-connecting-ip", "cf-connecting-ip"),
        ("x-forwarded-for:1", "x-forwarded-for"),
    ] {
        assert_eq!(
            key_of_bytes(source, peer, &[(header, b"198.51.100.9\xff")]).await,
            "unknown",
            "{source}"
        );
    }
}

#[tokio::test]
async fn a_missing_ip_is_logged_without_the_header_value() {
    let captured = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(captured.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    key_of(
        "cf-connecting-ip",
        Some("10.0.0.2:51000"),
        &[("cf-connecting-ip", "secret-value")],
    )
    .await;

    let log = captured.text();
    assert!(log.contains("client ip unavailable"), "{log}");
    assert!(log.contains(r#""level":"WARN""#), "{log}");
    assert!(!log.contains("secret-value"), "{log}");
}
