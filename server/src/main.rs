use std::net::SocketAddr;
use std::str::FromStr;
use std::convert::Infallible;

use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use clap::{AppSettings, Arg, Command};

mod session;
mod config;

async fn body_to_string(req: Request<Body>) -> String {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    String::from_utf8(body_bytes.to_vec()).unwrap()
}

async fn remote_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let upstream_addr = config::get(config::UPSTREAM_ADDR);
    let index_html = include_bytes!("../../webui_vanilla/dist/index.html");

    println!("Upstream address: {upstream_addr}");

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let reponse_str = String::from_utf8_lossy(index_html);
            Ok(Response::new(Body::from(reponse_str)))
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
    let mut app = Command::new("novnc-webrtc")
        .version("0.1.0")
        .author("Xuan Son NGUYEN <contact@ngxson.com>")
        .about("noVNC with WebRTC as transport layer. This application acts as a proxy to connect between Browser and VNC server (via TCP connection).\n\nIn short: Browser <== WebRTC ==> This app <== TCP ==> VNC server")
        .setting(AppSettings::DeriveDisplayOrder)
        .subcommand_negates_reqs(true)
        .arg(
            Arg::new("FULLHELP")
                .help("Prints more detailed help information")
                .long("help"),
        )
        .arg(
            Arg::new("listen")
                .takes_value(true)
                .default_value("0.0.0.0:6901")
                .long("listen")
                .short('l')
                .help("Address for server to listen."),
        )
        .arg(
            Arg::new("upstream")
                .takes_value(true)
                .default_value("127.0.0.1:5901")
                .long("upstream")
                .short('u')
                .help("Upstream addressof VNC server"),
        );
    let matches = app.clone().get_matches();
    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }
    if matches.is_present("listen") {
        let v: &str = matches.value_of("listen").unwrap();
        config::set(config::LISTEN_ADDR, String::from(v));
    }
    if matches.is_present("upstream") {
        let v: &str = matches.value_of("upstream").unwrap();
        config::set(config::UPSTREAM_ADDR, String::from(v));
    }
    let upstream_addr = config::get(config::UPSTREAM_ADDR);
    println!("Upstream address: {upstream_addr}");

    // main app
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
