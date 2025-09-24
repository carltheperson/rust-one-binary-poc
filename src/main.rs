use axum::{
    Router,
    body::Body,
    http::{Response, StatusCode, Uri, header},
    response::{Html, IntoResponse},
    routing::get,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "./frontend/build"]
#[allow_missing = true]
struct Assets;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/hello", get(hello))
        .fallback(static_handler);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> String {
    Assets::get("index.html")
        .unwrap()
        .data
        .to_vec()
        .as_slice()
        .as_ref()
        .len()
        .to_string()
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

    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "404").into_response(),
    }
}
