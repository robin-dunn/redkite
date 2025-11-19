use hyper::{
    Body, Client, Request, Response, Server, Uri,
    service::{make_service_fn, service_fn},
};
use std::convert::Infallible;

async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Extract what we need BEFORE moving req
    let original_headers = req.headers().clone();
    let method = req.method().clone();

    let host = original_headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<missing>");

    // Prevent recursive forwarding to self
    if host == "localhost:3000" || host == "127.0.0.1:3000" {
        return Ok(Response::builder()
            .status(400)
            .body(Body::from("Request to proxy itself not allowed"))
            .unwrap());
    }

    let path = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    // --- Minimal logging ---
    println!("--> {} http://{}{}", method, host, path);

    // Build outgoing URL
    let uri_str = format!("http://{}{}", host, path);
    let uri: Uri = uri_str.parse().unwrap();

    // Now we can consume the body safely
    let body = req.into_body();

    // Build new request
    let mut new_req = Request::builder()
        .method(method)
        .uri(uri)
        .body(body)
        .unwrap();

    *new_req.headers_mut() = original_headers;

    // Forward using the Hyper client
    let client = Client::new();
    client.request(new_req).await
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
