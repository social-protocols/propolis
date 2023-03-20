use axum::{
    body::{self, Empty, Full},
    extract::Path,
    response::{IntoResponse, Response},
};
use http::{header, HeaderValue, StatusCode};

pub async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    // https://benw.is/posts/serving-static-files-with-axum
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match crate::INCLUDED_STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .header(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=604800"),
            )
            .body(body::boxed(Full::from(file.contents())))
            .unwrap(),
    }
}
