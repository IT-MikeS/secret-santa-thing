use axum::{
    http::{header, Uri},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "www/"]
struct Assets;

static INDEX_HTML: &str = "index.html";

pub async fn handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return serve_index_html().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let body = axum::body::Body::from(content.data);
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        None => serve_index_html().await,
    }
}

async fn serve_index_html() -> Response {
    let index_html_asset = Assets::get(INDEX_HTML).unwrap();
    let body = axum::body::Body::from(index_html_asset.data);

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        .body(body)
        .unwrap()
}
