mod cli;
mod database;
mod isam;
mod net;
mod request;
mod tests;

use database::Database;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use websocket::sync::Server;

#[tokio::main]
async fn main() {
    // Parse arguments
    let args = cli::load_args();

    match args {
        // If no subcommand was specified, run the program regularly
        cli::Args::Main {
            database,
            no_tcp,
            no_ws,
            ws_cert,
            ws_key,
        } => {
            // Initialise the database to be mutable and thread-safe
            if let Ok(loaded_db) = isam::load(&database) {
                let db = Arc::new(RwLock::new(loaded_db));
                let isam_db_ref = db.clone();

                // Create a thread for each type of connection
                if !no_tcp {
                    let tcp_socket = TcpListener::bind("127.0.0.1:1337").await.unwrap();
                    let tcp_db_ref = db.clone();
                    tokio::spawn(async move {
                        net::tcp::handler(tcp_socket, &tcp_db_ref).await;
                    });
                }

                if !no_ws {
                    if let Ok(tls) = net::ws::init_tls(&ws_cert, &ws_key) {
                        let ws_socket = Server::bind_secure("127.0.0.1:1338", tls).unwrap();
                        let ws_db_ref = db.clone();
                        tokio::spawn(async move {
                            net::ws::handler(ws_socket, &ws_db_ref).await;
                        });
                    } else {
                        return println!("Unspecified or invalid TLS certificate. If you're not using WebSocket, pass the `--no-ws` argument to ignore.");
                    }
                }

                // Create a thread to asynchronously mirror the database to disk
                tokio::spawn(async move {
                    isam::mirror_handler(isam_db_ref, "database").await;
                });

                // Idles the main thread
                std::thread::park();
            } else {
                return println!("Unspecified or invalid database.");
            }
        }

        // If the create subcommand was specified, create a database
        cli::Args::Create { name } => {
            return create_database(&name);
        }
    }
}

/// Create a new database.
/// Executed when the user runs `jasondb create <database name>`.
fn create_database(name: &str) {
    let db = Database::new(name);
    isam::save(name, &db);
    println!("Empty database created.");
}
