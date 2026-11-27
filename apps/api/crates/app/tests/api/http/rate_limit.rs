use std::net::IpAddr;

use axum::http::{StatusCode, header};
use clinicore_app::router;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::support::{Answer, app_on, dispatch, from_peer, json_request, random_ip, state_on};

fn person(index: usize) -> Value {
    json!({
        "name": "Ana Souza",
        "email": format!("pessoa{index}@exemplo.com"),
        "password": "Clinica#2026",
    })
}

async fn sign_up_from(
    app: &axum::Router,
    peer: IpAddr,
    index: usize,
    headers: &[(&str, &str)],
) -> Answer {
    dispatch(
        app.clone(),
        from_peer(json_request("/users", &person(index), headers), peer),
    )
    .await
}

fn assert_limited(answer: &Answer) {
    assert_eq!(answer.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        answer.json()["type"],
        "tag:clinicore.com.br,2026:rate-limited"
    );
    assert_eq!(answer.json()["title"], "Too many requests");
    assert_eq!(answer.headers[header::CACHE_CONTROL], "no-store");
    let retry_after: u64 = answer.headers[header::RETRY_AFTER]
        .to_str()
        .expect("the header is text")
        .parse()
        .expect("the header is a number of seconds");
    assert!((1..=60).contains(&retry_after), "{retry_after}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_fourth_sign_up_from_one_ip_in_a_minute_is_refused(pool: PgPool) {
    let app = app_on(pool.clone());
    let peer = random_ip();

    for index in 0..3 {
        assert_eq!(
            sign_up_from(&app, peer, index, &[]).await.status,
            StatusCode::ACCEPTED
        );
    }
    assert_limited(&sign_up_from(&app, peer, 3, &[]).await);
    assert_eq!(
        sign_up_from(&app, random_ip(), 4, &[]).await.status,
        StatusCode::ACCEPTED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_request_the_origin_guard_refuses_spends_no_count(pool: PgPool) {
    let app = app_on(pool);
    let peer = random_ip();

    for index in 0..2 {
        let answer = sign_up_from(&app, peer, index, &[("origin", "https://outro.example")]).await;
        assert_eq!(answer.status, StatusCode::FORBIDDEN);
    }
    for index in 2..5 {
        assert_eq!(
            sign_up_from(&app, peer, index, &[]).await.status,
            StatusCode::ACCEPTED
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn behind_one_proxy_only_the_rightmost_forwarded_entry_counts(pool: PgPool) {
    let app = router(state_on(pool, &[("CLIENT_IP_SOURCE", "x-forwarded-for:1")]));
    let client = random_ip().to_string();

    for index in 0..4 {
        let forwarded = format!("1.1.1.{index}, {client}");
        let answer =
            sign_up_from(&app, random_ip(), index, &[("x-forwarded-for", &forwarded)]).await;
        if index < 3 {
            assert_eq!(answer.status, StatusCode::ACCEPTED);
        } else {
            assert_limited(&answer);
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn behind_cloudflare_the_connecting_ip_counts_whatever_the_forwarded_header_says(
    pool: PgPool,
) {
    let app = router(state_on(pool, &[("CLIENT_IP_SOURCE", "cf-connecting-ip")]));
    let client = random_ip().to_string();

    for index in 0..4 {
        let forwarded = format!("1.1.1.{index}");
        let headers = [
            ("cf-connecting-ip", client.as_str()),
            ("x-forwarded-for", forwarded.as_str()),
        ];
        let answer = sign_up_from(&app, random_ip(), index, &headers).await;
        if index < 3 {
            assert_eq!(answer.status, StatusCode::ACCEPTED);
        } else {
            assert_limited(&answer);
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn requests_without_the_ip_header_share_the_unknown_counter(pool: PgPool) {
    let state = state_on(pool, &[("CLIENT_IP_SOURCE", "cf-connecting-ip")]);
    let mut redis = state.redis.clone();
    redis::cmd("DEL")
        .arg("rate:sign-up:ip:unknown")
        .exec_async(&mut redis)
        .await
        .expect("redis answers");
    let app = router(state);

    for index in 0..3 {
        assert_eq!(
            sign_up_from(&app, random_ip(), index, &[]).await.status,
            StatusCode::ACCEPTED
        );
    }
    assert_limited(&sign_up_from(&app, random_ip(), 3, &[]).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_ipv6_addresses_of_one_64_share_a_counter(pool: PgPool) {
    let app = app_on(pool);
    let network: [u16; 4] = rand::random();
    let address = |host: u16| -> IpAddr {
        IpAddr::from([
            network[0], network[1], network[2], network[3], 0, 0, 0, host,
        ])
    };

    for index in 0..3 {
        let answer = sign_up_from(&app, address(index as u16 + 1), index, &[]).await;
        assert_eq!(answer.status, StatusCode::ACCEPTED);
    }
    assert_limited(&sign_up_from(&app, address(99), 3, &[]).await);
}

async fn confirm_from(app: &axum::Router, peer: IpAddr) -> Answer {
    let body = json!({ "token": "a".repeat(43) });
    dispatch(
        app.clone(),
        from_peer(
            json_request("/email-verifications/confirmation", &body, &[]),
            peer,
        ),
    )
    .await
}

async fn fill_network(state: &clinicore_app::AppState, network: &str) {
    let mut redis = state.redis.clone();
    redis::cmd("SET")
        .arg(format!("rate:email-confirmation:network:{network}"))
        .arg(300)
        .arg("EX")
        .arg(60)
        .exec_async(&mut redis)
        .await
        .expect("redis answers");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_fourth_link_request_from_one_ip_in_a_minute_is_refused(pool: PgPool) {
    let app = app_on(pool);
    let peer = random_ip();
    let request = |index: usize| {
        let body = json!({ "email": format!("pessoa{index}@exemplo.com") });
        from_peer(json_request("/email-verifications", &body, &[]), peer)
    };

    for index in 0..3 {
        let answer = dispatch(app.clone(), request(index)).await;
        assert_eq!(answer.status, StatusCode::ACCEPTED);
    }
    assert_limited(&dispatch(app, request(3)).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_eleventh_confirmation_from_one_ip_in_a_minute_is_refused(pool: PgPool) {
    let app = app_on(pool);
    let peer = random_ip();

    for _ in 0..10 {
        assert_eq!(
            confirm_from(&app, peer).await.status,
            StatusCode::BAD_REQUEST
        );
    }
    assert_limited(&confirm_from(&app, peer).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_full_ipv4_24_refuses_the_confirmation_of_every_address_in_it(pool: PgPool) {
    let state = state_on(pool, &[]);
    let [first, second, third]: [u8; 3] = rand::random();
    fill_network(&state, &format!("{first}.{second}.{third}.0/24")).await;
    let app = router(state);

    let inside = IpAddr::from([first, second, third, 77]);
    let outside = IpAddr::from([first, second, third.wrapping_add(1), 77]);

    assert_limited(&confirm_from(&app, inside).await);
    assert_eq!(
        confirm_from(&app, outside).await.status,
        StatusCode::BAD_REQUEST
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_full_ipv6_48_refuses_the_confirmation_of_every_address_in_it(pool: PgPool) {
    let state = state_on(pool, &[]);
    let network: [u16; 3] = rand::random();
    let address = |third: u16, host: u16| -> IpAddr {
        IpAddr::from([network[0], network[1], third, 0xbbbb, 0, 0, 0, host])
    };
    let range = std::net::Ipv6Addr::from([network[0], network[1], network[2], 0, 0, 0, 0, 0]);
    fill_network(&state, &format!("{range}/48")).await;
    let app = router(state);

    assert_limited(&confirm_from(&app, address(network[2], 1)).await);
    assert_eq!(
        confirm_from(&app, address(network[2].wrapping_add(1), 1))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
}
