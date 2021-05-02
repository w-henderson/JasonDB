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
pub async fn handler(listener: TcpListener, db: &Arc<RwLock<Database>>) {
    loop {
        // Continously accept connections synchronously
        match listener.accept().await {
            Ok((socket, _)) => {
                let db_ref = db.clone();

                // Accept each connection then passes management of the connection to another thread.
                // This thread continously listens for requests and responds to them.
                tokio::spawn(async move {
                    let mut lines = Framed::new(socket, LinesCodec::new());
                    while let Some(result) = lines.next().await {
                        match result {
                            Ok(line) => {
                                // Parse and execute the request
                                let request = request::parse(&line);
                                let response = request::execute(request, &db_ref);

                                // Send the response
                                lines.send(response.to_json()).await.unwrap();
                            }
                            Err(e) => {
                                match e {
                                    tokio_util::codec::LinesCodecError::Io(_) => {
                                        return;
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                });
            }
            Err(_) => (),
        }
    }
}
