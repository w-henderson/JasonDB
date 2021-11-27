//! Manages the CLI, argument parsing and logging.

use clap::{load_yaml, App, AppSettings};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    time::SystemTime,
};

/// Represents arguments passed to the program.
pub enum Args {
    Main {
        database: String,
        no_tcp: bool,
        no_ws: bool,
        tcp_port: u16,
        ws_port: u16,
        ws_cert: String,
        ws_key: String,
        mirror_interval: u64,
        log_config: LogConfig,
    },
    Create {
        name: String,
    },
    Extract {
        path: String,
    },
    Error {
        message: String,
    },
}

#[derive(Clone)]
pub struct LogConfig {
    quiet: bool,
    file: Option<String>,
}

impl LogConfig {
    pub fn force(&self) -> Self {
        Self {
            quiet: false,
            file: self.file.clone(),
        }
    }

    pub fn default() -> Self {
        Self {
            quiet: false,
            file: None,
        }
    }
}

/// Loads the arguments that were passed to the program.
/// Returns an enum representing the command and its parameters.
pub fn load_args() -> Args {
    let yaml = load_yaml!("../cli/en-gb.yaml");
    let app = App::from(yaml)
        .setting(AppSettings::SubcommandsNegateReqs)
        .setting(AppSettings::ArgRequiredElseHelp);

    let matches = app.get_matches();

    if let Some(subcommand) = matches.subcommand_matches("create") {
        Args::Create {
            name: subcommand.value_of("name").unwrap().to_string(),
        }
    } else if let Some(subcommand) = matches.subcommand_matches("extract") {
        Args::Extract {
            path: subcommand.value_of("path").unwrap().to_string(),
        }
    } else {
        if let Some(logfile) = matches.value_of("logfile") {
            if File::create(logfile).is_err() {
                return Args::Error {
                    message: "Log file could not be created.".to_string(),
                };
            }
        }

        Args::Main {
            database: matches.value_of("DATABASE").unwrap().to_string(),
            no_tcp: matches.is_present("no-tcp"),
            no_ws: matches.is_present("no-ws"),
            tcp_port: matches.value_of_t("tcp-port").unwrap_or(1337),
            ws_port: matches.value_of_t("ws-port").unwrap_or(1338),
            ws_cert: matches.value_of("cert").unwrap_or("").to_string(),
            ws_key: matches.value_of("key").unwrap_or("").to_string(),
            mirror_interval: matches.value_of_t("interval").unwrap_or(0),
            log_config: LogConfig {
                quiet: matches.is_present("quiet"),
                file: matches.value_of("logfile").map(String::from),
            },
        }
    }
}

pub fn log(message: &str, config: &LogConfig) {
    if !config.quiet {
        println!("{}", message);
    }

    if let Some(file_name) = &config.file {
        let mut file = OpenOptions::new().append(true).open(file_name).unwrap();
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
            + " ";
        file.write(time.as_bytes()).unwrap();
        file.write(message.as_bytes()).unwrap();
        file.write(b"\n").unwrap();
    }
}
