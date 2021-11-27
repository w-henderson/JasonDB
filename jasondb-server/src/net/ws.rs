//! Manages WebSocket connections and TLS.

use crate::cli::LogConfig;
use crate::request;

use jasondb::database::Database;

use dotenv::var;
use native_tls::{Identity, TlsAcceptor};
use parking_lot::RwLock;
use std::{fs::File, io::Read, sync::Arc};
use std::{net::TcpListener, thread};
use websocket::{server::WsServer, OwnedMessage};

/// Initialises TLS by reading a key from a file and returning it.
/// Reads `CERT` (path to certificate) and `KEY` (password to certificate) from a `.env` file.
/// This is required to use WebSockets over the `wss://` protocol.
///
/// ## Generating a locally-trusted key
/// You need `mkcert` (https://github.com/FiloSottile/mkcert) to do this.
/// Run the following commands in the program directory to generate a key:
/// ```bash
/// $ mkcert -install # instruct the system to trust the mkcert certificate authority
/// $ mkcert -pkcs12 localhost # generate a PKCS12 certificate
/// ```
/// You then need to set the `.env` file configuration as follows. The key defaults to `changeit` if not changed.
/// ```bash
/// CERT=<path to certificate>
/// KEY=<key>
/// ```
///
/// TODO: Implement error handling.
#[rustfmt::skip]
pub fn init_tls(path: &str, key: &str) -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    // Attempts to read certificate information from `.env` if not specified
    let path = if path == "" { var("CERT")? } else { path.to_string() };
    let key = if key == "" { var("KEY")? } else { key.to_string() };

    // Opens and reads the certificate file
    let mut file = File::open(path)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes)?;

    // Parse the file into a TLS acceptor
    let identity = Identity::from_pkcs12(&bytes, &key)?;
    Ok(TlsAcceptor::new(identity)?)
}

/// Handles WebSocket connections asynchronously.
/// Creates a new thread for each individual connection, but individual messages are handled synchronously inside that thread.
pub async fn handler(
    server: WsServer<TlsAcceptor, TcpListener>,
    db: &Arc<RwLock<Database>>,
    config: LogConfig,
) {
    crate::cli::log(
        &format!(
            "[WS]   Server listening at 127.0.0.1:{}",
            server.local_addr().unwrap().port()
        ),
        &config,
    );

    // Synchronously accept connections as they come in
    for request in server.filter_map(Result::ok) {
        let db_ref = db.clone();
        let config_clone = config.clone();

        // Create a new thread for managing two-way communication with the client.
        // Messages are responded to synchronously in this thread.
        thread::spawn(move || {
            let mut client = request.accept().unwrap();
            let ip = client.peer_addr().unwrap().ip().to_string();
            crate::cli::log(&format!("[WS]   New connection from {}", ip), &config_clone);

            loop {
                let msg = client.recv_message().unwrap();

                match msg {
                    OwnedMessage::Text(text) => {
                        // If the message is in the format `ID <some ID code here> <request>`,
                        // then we echo the ID back with the response so it can be tracked client-side.
                        if &text[0..3] != "ID " {
                            // Parses and executes the request
                            let request = request::parse(&text);
                            let response = request::execute(request, &db_ref);
                            let json_message = OwnedMessage::Text(response.to_json());

                            // Sends the response
                            client.send_message(&json_message).unwrap();
                        } else {
                            if let Some(request_start) = &text[3..].find(" ") {
                                // Parses and executes the request
                                let request = request::parse(&text[request_start + 4..]);
                                let response = request::execute(request, &db_ref);
                                let json_message = OwnedMessage::Text(format!(
                                    "ID {} {}",
                                    &text[3..*request_start + 3],
                                    response.to_json()
                                ));

                                // Sends the response
                                client.send_message(&json_message).unwrap();
                            } else {
                                client
                                    .send_message(&OwnedMessage::Text(
                                        r#"{"status": "error", "message": "Malformed ID"}"#
                                            .to_string(),
                                    ))
                                    .unwrap();
                            }
                        }

                        crate::cli::log(&format!("[WS]   {}: {}", ip, text), &config_clone);
                    }

                    OwnedMessage::Close(_) => {
                        break;
                    }

                    _ => (),
                }
            }
        });
    }
}