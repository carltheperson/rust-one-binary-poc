use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode, Uri, header},
    response::{Html, IntoResponse},
    routing::get,
};
use axum_server::bind;
use rust_embed::Embed;
use rustls_acme::tower::TowerHttp01ChallengeService;
use rustls_acme::{AcmeConfig, caches::DirCache};
use rustls_acme::{UseChallenge::Http01, futures_rustls::rustls};
use std::{
    net::{Ipv6Addr, SocketAddr},
    sync::Arc,
};
use tokio::try_join;
use tokio_stream::StreamExt;

mod db;
use crate::db::{ensure_item, get_item, open_db};
use rust_one_binary_poc::Item;

#[derive(Embed)]
#[folder = "./frontend/build"]
#[allow_missing = true]
struct Assets;

#[derive(Clone)]
struct AppState {
    db: Arc<sled::Db>,
}

#[tokio::main]
async fn main() {
    let db = open_db();

    // seed a few mock items
    seed_mock_items(&db);

    let root_cert = include_bytes!("../pebble.minica.pem");
    let mut root_store = rustls::RootCertStore::empty();
    let cert = rustls_pemfile::certs(&mut root_cert.as_slice())
        .next()
        .expect("Failed to parse certificate")
        .expect("No certificate found");
    root_store
        .add(cert)
        .expect("Failed to add certificate to root store");
    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let mut acme_state = AcmeConfig::new(vec!["app.test".to_string()])
        .cache_option(Some(DirCache::new("./data/acme")))
        .directory("https://localhost:14000/dir")
        .client_tls_config(Arc::new(client_config))
        .challenge_type(Http01)
        .state();

    let acceptor = acme_state.axum_acceptor(acme_state.default_rustls_config());
    let acme_challenge_tower_service: TowerHttp01ChallengeService =
        acme_state.http01_challenge_tower_service();

    tokio::spawn(async move {
        loop {
            match acme_state.next().await.unwrap() {
                Ok(ok) => println!("event: {:?}", ok),
                Err(err) => println!("error: {:?}", err),
            }
        }
    });

    let state = AppState { db: Arc::new(db) };

    let app = Router::new()
        .route("/api/items", get(list_items))
        .route("/api/items/{id}", get(single_item))
        .with_state(state)
        .route_service(
            "/.well-known/acme-challenge/{challenge_token}",
            acme_challenge_tower_service,
        )
        .fallback(static_handler);

    let http_addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 80));
    let https_addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, 443));

    let http_future = bind(http_addr).serve(app.clone().into_make_service());
    let https_future = bind(https_addr)
        .acceptor(acceptor)
        .serve(app.into_make_service());

    try_join!(https_future, http_future).unwrap();
}

async fn static_handler(uri: Uri, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    let accept = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let immutable = path.starts_with("_app/immutable/");

    let candidates = if immutable {
        let mut v = Vec::with_capacity(3);
        if accept.contains("br") {
            v.push(format!("{path}.br"));
        }
        if accept.contains("gzip") {
            v.push(format!("{path}.gz"));
        }
        v.push(path.to_string());
        v
    } else {
        vec![path.to_string()]
    };

    if let Some((name, content)) = candidates
        .into_iter()
        .find_map(|p| Assets::get(&p).map(|c| (p, c)))
    {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let mut builder = Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::VARY, "Accept-Encoding");

        if immutable {
            builder = builder.header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
        }

        for (suffix, enc) in [(".br", "br"), (".gz", "gzip")] {
            if name.ends_with(suffix) {
                builder = builder.header(header::CONTENT_ENCODING, enc);
                break;
            }
        }

        return builder.body(Body::from(content.data)).unwrap();
    }

    // SPA fallback
    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "404").into_response(),
    }
}

async fn list_items(State(state): State<AppState>) -> Json<Vec<Item>> {
    let items = db::list_items(&state.db);
    Json(items)
}

async fn single_item(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    match get_item(&state.db, id) {
        Some(item) => Json(item).into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

fn seed_mock_items(db: &sled::Db) {
    let items = vec![
        Item {
            id: 1,
            name: "Item 1".to_string(),
        },
        Item {
            id: 2,
            name: "Item 2".to_string(),
        },
        Item {
            id: 3,
            name: "Item 3".to_string(),
        },
    ];
    for item in items.iter() {
        ensure_item(db, item);
    }
}
