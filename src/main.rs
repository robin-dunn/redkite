use hyper::{
    Body, Client, Request, Response, Server,
    service::{make_service_fn, service_fn},
};
use hyper::body::to_bytes;
use hyper_tls::HttpsConnector;
use std::convert::Infallible;

async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Extract necessary info before moving req
    let original_headers = req.headers().clone();
    let method = req.method().clone();
    let path = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    let host = original_headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<missing>");

    // Prevent infinite recursion
    if host == "localhost:3000" || host == "127.0.0.1:3000" {
        return Ok(Response::builder()
            .status(400)
            .body(Body::from("Request to proxy itself not allowed"))
            .unwrap());
    }

    println!("--> {} http://{}{}", method, host, path);

    // Determine scheme: default to https
    let scheme = if path.starts_with("http://") || path.starts_with("https://") {
        ""
    } else {
        "https://"
    };

    let uri: hyper::Uri = format!("{}{}{}", scheme, host, path).parse().unwrap();

    // Build outgoing request
    let mut new_req = Request::builder()
        .method(method)
        .uri(uri)
        .body(req.into_body())
        .unwrap();

    *new_req.headers_mut() = original_headers;

    // HTTPS client
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    // Send request
    let resp = client.request(new_req).await?;

    // Log response body (small responses only)
    let (parts, body) = resp.into_parts();
    let body_bytes = to_bytes(body).await?;
    println!("<-- Response body: {}", String::from_utf8_lossy(&body_bytes));

    // Reconstruct response for client
    Ok(Response::from_parts(parts, Body::from(body_bytes)))
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(proxy))
    });

    println!("Proxy running at http://127.0.0.1:3000");

    Server::bind(&addr)
        .serve(make_svc)
        .await
        .unwrap();
}