use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{ALLOW, CONTENT_TYPE, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use iconator::{get_icon_for_file, get_icon_for_folder};
use tokio::net::TcpListener;

#[derive(rust_embed::Embed)]
#[folder = "icons/"]
#[include = "*.svg"]
struct Icons;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("Listening on http://127.0.0.1:3000");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(error) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service_fn(handle_request))
                .await
            {
                eprintln!("Connection error: {error}");
            }
        });
    }
}

async fn handle_request<B>(
    request: Request<B>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let path = request.uri().query().and_then(|query| {
        form_urlencoded::parse(query.as_bytes())
            .find_map(|(key, value)| (key == "path").then_some(value))
    });

    match (request.uri().path(), request.method(), path.as_deref()) {
        ("/icons/file" | "/icons/folder", method, _) if method != Method::GET => {
            Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(ALLOW, HeaderValue::from_static("GET"))
                .body(Full::default())
        }
        ("/icons/file" | "/icons/folder", _, None | Some("")) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::default()),
        ("/icons/file", _, Some(path)) => {
            let Some(svg) = get_icon_for_file(path).and_then(|id| Icons::get(&format!("{id}.svg")))
            else {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::default());
            };

            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"))
                .body(Full::new(Bytes::from_owner(svg.data)))
        }
        ("/icons/folder", _, Some(path)) => {
            let Some(svg) =
                get_icon_for_folder(path).and_then(|id| Icons::get(&format!("{id}.svg")))
            else {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::default());
            };

            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"))
                .body(Full::new(Bytes::from_owner(svg.data)))
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test(flavor = "current_thread")]
    async fn serves_icon_svg() {
        let cases: &[(&str, &[u8])] = &[
            (
                "/icons/folder?path=src",
                include_bytes!("../icons/1054.svg"),
            ),
            (
                "/icons/folder?path=my+project%2F%73rc",
                include_bytes!("../icons/1054.svg"),
            ),
            (
                "/icons/folder?other=ignored&path=src",
                include_bytes!("../icons/1054.svg"),
            ),
            (
                "/icons/file?path=main.rs",
                include_bytes!("../icons/525.svg"),
            ),
            (
                "/icons/file?path=my+project%2Fmain%2Ers",
                include_bytes!("../icons/525.svg"),
            ),
            (
                "/icons/file?path=package.json",
                include_bytes!("../icons/381.svg"),
            ),
        ];
        for &(uri, expected) in cases {
            let request = Request::builder().uri(uri).body(()).unwrap();
            let response = handle_request(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{uri}");
            assert_eq!(response.headers()[CONTENT_TYPE], "image/svg+xml");
            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body.as_ref(), expected, "{uri}");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn handles_missing_and_unknown_paths() {
        for (uri, status) in [
            ("/icons/folder", StatusCode::BAD_REQUEST),
            ("/icons/folder?", StatusCode::BAD_REQUEST),
            ("/icons/folder?path=", StatusCode::BAD_REQUEST),
            ("/icons/folder?other=src", StatusCode::BAD_REQUEST),
            ("/icons/folder?path=unknown-folder", StatusCode::NOT_FOUND),
            ("/icons/folder?path=%2F", StatusCode::NOT_FOUND),
            ("/icons/file", StatusCode::BAD_REQUEST),
            ("/icons/file?path=", StatusCode::BAD_REQUEST),
            ("/icons/file?path=&path=main.rs", StatusCode::BAD_REQUEST),
            ("/icons/file?other=main.rs", StatusCode::BAD_REQUEST),
            ("/icons/file?path=unknown-file", StatusCode::NOT_FOUND),
            (
                "/icons/file?path=test.unknown-extension",
                StatusCode::NOT_FOUND,
            ),
            ("/icons/file?path=%2F", StatusCode::NOT_FOUND),
            ("/", StatusCode::NOT_FOUND),
        ] {
            let request = Request::builder().uri(uri).body(()).unwrap();
            let response = handle_request(request).await.unwrap();
            assert_eq!(response.status(), status, "{uri}");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rejects_other_methods() {
        for uri in [
            "/icons/folder?path=src",
            "/icons/file?path=main.rs",
            "/icons/folder",
            "/icons/file",
        ] {
            let request = Request::builder()
                .method(Method::POST)
                .uri(uri)
                .body(())
                .unwrap();
            let response = handle_request(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED, "{uri}");
            assert_eq!(response.headers()[ALLOW], "GET");
        }

        let request = Request::builder()
            .method(Method::POST)
            .uri("/unknown")
            .body(())
            .unwrap();
        let response = handle_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(!response.headers().contains_key(ALLOW));
    }
}
