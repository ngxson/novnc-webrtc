use std::sync::Arc;
use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::Result;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub(crate) async fn start_webrtc_session(sdp: String, upstream_addr: String) -> Result<String> {
    // Create MediaEngine instance
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting");
            let _ = done_tx.try_send(());
        }

        Box::pin(async {})
    }));

    // Register data channel creation handling
    peer_connection.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let d_label = d.label().to_owned();
        let d_id = d.id();
        let d_upstream_addr = upstream_addr.to_owned();
        println!("New DataChannel {d_label} {d_id}");

        // Register channel opening handling
        Box::pin(async move {
            // Create connection to upstream
            let saddr = SocketAddr::from_str(d_upstream_addr.as_str()).unwrap();
            let stream_result = TcpStream::connect(saddr).await;
            if let Err(_err) = &stream_result {
                eprintln!("Failed to connect to upstream {d_upstream_addr}");
                return;
            }
            let stream = stream_result.unwrap();
            let (mut reader, writer) = stream.into_split();
            let writer = Arc::new(Mutex::new(writer));

            let d2 = Arc::clone(&d);
            let d_label2 = d_label.clone();
            let d_id2 = d_id;
            d.on_close(Box::new(move || {
                println!("Data channel closed");
                // TODO: also close TCP connection
                Box::pin(async { })
            }));

            d.on_open(Box::new(move || {
                println!("Data channel '{d_label2}'-'{d_id2}' open.");

                Box::pin(async move {
                    let mut buffer = [0; 1024];
                    loop {
                        let n_result = reader.read(&mut buffer).await;
                        if let Err(_err) = &n_result {
                            eprintln!("Failed to read from upstream {d_upstream_addr}, reason: {_err}");
                            break;
                        }
                        let n = n_result.unwrap();
                        if n == 0 {
                            eprintln!("Received 0 bytes from upstream {d_upstream_addr}, disconnect");
                            break;
                        }
                        // println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
                        let imm_buffer = bytes::Bytes::copy_from_slice(&mut buffer[..n]);
                        let send_result = d2.send(&imm_buffer).await;
                        if let Err(_err) = send_result {
                            eprintln!("Failed to send to webrtc peer, reason: {_err}");
                            break;
                        }
                    }
                })
            }));

            // Register text message handling
            d.on_message(Box::new(move |msg: DataChannelMessage| {
                // let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                // println!("Message from DataChannel '{d_label}': '{msg_str}'");
                let msg_bytes = msg.data.to_owned();
                let writer_clone = Arc::clone(&writer);
                Box::pin(async move {
                    let mut writer_locked = writer_clone.lock().await;
                    let write_result = writer_locked.write_all(&msg_bytes).await;
                    if let Err(_err) = write_result {
                        eprintln!("Failed to write to webrtc peer, reason: {_err}");
                        return;
                    }
                })
            }));
        })
    }));

    let offer = serde_json::from_str::<RTCSessionDescription>(&sdp).unwrap();
    peer_connection.set_remote_description(offer).await.unwrap();
    let answer = peer_connection.create_answer(None).await?;
    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer local_desc
    let mut answer: String = "".to_string();
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc).unwrap();
        // println!("{json_str}");
        println!("Generated SDP answer, sending back to client...");
        answer = json_str.to_string();
    } else {
        println!("generate local_description failed!");
    }

    tokio::spawn(async move {
        tokio::select! {
            _ = done_rx.recv() => {
                println!("Received done signal!");
            }
        };
    
        peer_connection.close().await?; 

        Ok::<(), webrtc::Error>(())
    });

    Ok(answer)
}