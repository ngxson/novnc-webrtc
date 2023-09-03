use std::net::SocketAddr;
use std::str::FromStr;
use std::convert::Infallible;

use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};

mod session;
mod config;

async fn body_to_string(req: Request<Body>) -> String {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    String::from_utf8(body_bytes.to_vec()).unwrap()
}

async fn remote_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let upstream_addr = config::get(config::UPSTREAM_ADDR);
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            Ok(Response::new(Body::from("Hello, World!")))
        },
        (&Method::POST, "/sdp") => {
            let payload: String = body_to_string(req).await;
            // println!("{:?}", payload);
            println!("Received SDP, starting WebRTC session...");
            let answer = session::start_webrtc_session(payload, upstream_addr).await.unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, PUT, DELETE, OPTIONS")
                .body(Body::from(answer))
                .unwrap())
        },
        (&Method::OPTIONS, "/sdp") => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, PUT, DELETE, OPTIONS")
                .body(Body::from(""))
                .unwrap())
        }
        _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not Found"))
                    .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let addr_str = config::get(config::LISTEN_ADDR);
    let addr = SocketAddr::from_str(&config::get(config::LISTEN_ADDR)).unwrap();
    let service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(remote_handler)) });
    let server = Server::bind(&addr).serve(service);
    println!("Listening on {addr_str}");
    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {e}");
    }

    Ok(())
}
