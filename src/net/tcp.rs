//! Manages TCP connections.

use crate::database::Database;
use crate::request;
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LinesCodec};

/// Handles TCP connections asynchronously.
/// Creates a new thread for each individual connection, but individual requests are handled synchronously inside that thread.
pub async fn handler(listener: TcpListener, db: &Arc<RwLock<Database>>, quiet: bool) {
    loop {
        // Continously accept connections synchronously
        match listener.accept().await {
            Ok((socket, _)) => {
                let db_ref = db.clone();
                let ip = socket.peer_addr().unwrap().ip().to_string();
                crate::cli::log(format!("[TCP]  New connection from {}", ip), quiet);

                // Accept each connection then passes management of the connection to another thread.
                // This thread continously listens for requests and responds to them.
                tokio::spawn(async move {
                    let mut lines = Framed::new(socket, LinesCodec::new());
                    while let Some(result) = lines.next().await {
                        match result {
                            Ok(line) => {
                                // Requests can be joined together in one packet with the string "THEN".
                                // For example, "GET user1 FROM users THEN GET user2 FROM users"
                                // They should be processed separately but the result returned together.
                                // The result is joined with the "&" character.
                                let mut responses: Vec<String> = Vec::new();
                                for line_part in line.split(" THEN ") {
                                    // Parse and execute the request
                                    let request = request::parse(line_part);
                                    let response = request::execute(request, &db_ref);
                                    responses.push(response.to_json());
                                }

                                // Send the response(s)
                                let response = if responses.len() == 1 {
                                    responses[0].clone()
                                } else {
                                    format!("[{}]", responses.join(","))
                                };
                                lines.send(&response).await.unwrap();

                                crate::cli::log(format!("[TCP]  {}: {}", ip, line), quiet);
                            }
                            Err(e) => match e {
                                tokio_util::codec::LinesCodecError::Io(_) => {
                                    return;
                                }
                                _ => (),
                            },
                        }
                    }
                });
            }
            Err(_) => (),
        }
    }
}
