mod cli;
mod database;
mod extract;
mod isam;
mod net;
mod request;
mod tests;

use database::Database;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU8, Ordering};
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
            mirror_interval,
            quiet,
        } => {
            // Initialise the database to be mutable and thread-safe
            if let Ok(loaded_db) = isam::load(&database) {
                let db = Arc::new(RwLock::new(loaded_db));
                let isam_db_ref = db.clone();

                // Initialise a variable to store the state of the application
                // 0 - running, 1 - stopping, 2 - safe to terminate
                let application_state = Arc::new(AtomicU8::new(0));

                // Create a thread for each type of connection
                if !no_tcp {
                    let tcp_socket = TcpListener::bind("0.0.0.0:1337").await.unwrap();
                    let tcp_db_ref = db.clone();
                    tokio::spawn(async move {
                        net::tcp::handler(tcp_socket, &tcp_db_ref, quiet).await;
                    });
                }

                if !no_ws {
                    if let Ok(tls) = net::ws::init_tls(&ws_cert, &ws_key) {
                        let ws_socket = Server::bind_secure("0.0.0.0:1338", tls).unwrap();
                        let ws_db_ref = db.clone();
                        tokio::spawn(async move {
                            net::ws::handler(ws_socket, &ws_db_ref, quiet).await;
                        });
                    } else {
                        return println!("[ERR]  Unspecified or invalid TLS certificate. If you're not using WebSocket, pass the `--no-ws` argument to ignore.");
                    }
                }

                // Create a thread to asynchronously mirror the database to disk
                let isam_application_state = application_state.clone();
                tokio::spawn(async move {
                    isam::mirror_handler(
                        isam_db_ref,
                        &database,
                        mirror_interval,
                        isam_application_state,
                        quiet,
                    )
                    .await;
                });

                ctrlc::set_handler(move || {
                    application_state.store(1, Ordering::SeqCst);
                    println!("[DISK] Waiting for next save to complete...");

                    // Wait for the state to change to 2 (safe to terminate).
                    // This change happens in the ISAM thread and can take up to 5 seconds.
                    while application_state.load(Ordering::SeqCst) == 1 {}

                    // Safely exit the program
                    println!("[INFO] Exiting the program.");
                    std::process::exit(0);
                })
                .expect("[ERR]  Couldn't set exit handler");

                println!("[INFO] JasonDB active and accessible at 127.0.0.1:1337");

                // Idles the main thread
                std::thread::park();
            } else {
                return println!("[ERR]  Unspecified or invalid database.");
            }
        }

        // If the create subcommand was specified, create a database
        cli::Args::Create { name } => {
            return create_database(&name);
        }

        // If the extract command was specified, run the extraction tool
        cli::Args::Extract { path } => {
            return if let Ok(()) = extract::extract(&path) {
                println!("[INFO] Database extracted.")
            } else {
                println!("[ERR]  An error occurred.")
            }
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
