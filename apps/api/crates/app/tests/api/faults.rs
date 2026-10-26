use std::sync::{Arc, Mutex};

use axum::Router;
use clinicore_app::GoogleEndpoints;
use clinicore_core::redis::Redis;
use redis::{ConnectionAddr, IntoConnectionInfo};
use sqlx::PgPool;
use sqlx::postgres::{PgPoolOptions, PgSslMode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};

use crate::support::config_with;

const CARRIED_BYTES: usize = 128;

#[derive(Clone, Default)]
pub struct Fault(Arc<Mutex<Option<Cut>>>);

struct Cut {
    command: &'static [u8],
    skip: usize,
}

impl Fault {
    pub fn cut_at(&self, command: &'static str, skip: usize) {
        *self.0.lock().expect("an unpoisoned fault") = Some(Cut {
            command: command.as_bytes(),
            skip,
        });
    }

    fn trips(&self, carried: &[u8], read: &[u8]) -> bool {
        let mut armed = self.0.lock().expect("an unpoisoned fault");
        let Some(cut) = armed.as_mut() else {
            return false;
        };
        let sent = [carried, read].concat();
        let seen = sent
            .windows(cut.command.len())
            .enumerate()
            .filter(|(start, window)| {
                start + cut.command.len() > carried.len() && *window == cut.command
            })
            .count();
        if seen > cut.skip {
            *armed = None;
            return true;
        }
        cut.skip -= seen;
        false
    }
}

pub struct FaultyApp {
    pub app: Router,
    pub database: Fault,
    pub redis: Fault,
}

pub async fn faulty_app(pool: &PgPool) -> FaultyApp {
    faulty_app_with_google(pool, GoogleEndpoints::production()).await
}

pub async fn faulty_app_with_google(pool: &PgPool, google: GoogleEndpoints) -> FaultyApp {
    let database = Fault::default();
    let options = pool.connect_options();
    let database_port = proxy(
        options.get_host().to_string(),
        options.get_port(),
        database.clone(),
    )
    .await;
    let faulty_pool = PgPoolOptions::new().connect_lazy_with(
        (*options)
            .clone()
            .host("127.0.0.1")
            .port(database_port)
            .ssl_mode(PgSslMode::Disable)
            .statement_cache_capacity(0),
    );

    let redis = Fault::default();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    let redis_info = redis_url
        .into_connection_info()
        .expect("a Redis connection string");
    let ConnectionAddr::Tcp(redis_host, redis_port) = redis_info.addr().clone() else {
        panic!("a TCP Redis");
    };
    let redis_port = proxy(redis_host, redis_port, redis.clone()).await;

    let mut config = config_with(&[]);
    config.redis_url = redis_info.set_addr(ConnectionAddr::Tcp("127.0.0.1".into(), redis_port));
    let state = clinicore_app::AppState::new(
        &config,
        faulty_pool,
        Redis::connect_lazy(&config),
        clinicore_app::GoogleClient::new(&config, google),
    );
    FaultyApp {
        app: clinicore_app::app(&config, state),
        database,
        redis,
    }
}

async fn proxy(upstream_host: String, upstream_port: u16, fault: Fault) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free local port");
    let port = listener.local_addr().expect("a bound address").port();
    tokio::spawn(async move {
        while let Ok((client, _)) = listener.accept().await {
            let upstream = TcpStream::connect((upstream_host.as_str(), upstream_port))
                .await
                .expect("the upstream behind the proxy");
            let fault = fault.clone();
            tokio::spawn(async move {
                let (from_client, mut to_client) = client.into_split();
                let (mut from_upstream, to_upstream) = upstream.into_split();
                tokio::select! {
                    _ = tokio::io::copy(&mut from_upstream, &mut to_client) => {}
                    () = forward(from_client, to_upstream, fault) => {}
                }
            });
        }
    });
    port
}

async fn forward(mut from_client: OwnedReadHalf, mut to_upstream: OwnedWriteHalf, fault: Fault) {
    let mut buffer = vec![0; 64 * 1024];
    let mut carried = Vec::new();
    loop {
        let read = match from_client.read(&mut buffer).await {
            Ok(0) | Err(_) => return,
            Ok(read) => read,
        };
        if fault.trips(&carried, &buffer[..read]) {
            return;
        }
        if to_upstream.write_all(&buffer[..read]).await.is_err() {
            return;
        }
        carried.extend_from_slice(&buffer[..read]);
        carried.drain(..carried.len().saturating_sub(CARRIED_BYTES));
    }
}
