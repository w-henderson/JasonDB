use clap::{load_yaml, App, AppSettings};

/// Represents arguments passed to the program.
pub enum Args {
    Main {
        database: String,
        no_tcp: bool,
        no_ws: bool,
        ws_cert: String,
        ws_key: String,
    },
    Create {
        name: String,
    },
}

/// Loads the arguments that were passed to the program.
/// Returns an enum representing the command and its parameters.
pub fn load_args() -> Args {
    let yaml = load_yaml!("../cli/en-gb.yaml");
    let app = App::from(yaml).setting(AppSettings::SubcommandsNegateReqs);

    let matches = app.get_matches();

    if let Some(subcommand) = matches.subcommand_matches("create") {
        Args::Create {
            name: subcommand.value_of("name").unwrap().to_string(),
        }
    } else {
        Args::Main {
            database: matches.value_of("DATABASE").unwrap().to_string(),
            no_tcp: matches.is_present("no-tcp"),
            no_ws: matches.is_present("no-ws"),
            ws_cert: matches.value_of("cert").unwrap_or("").to_string(),
            ws_key: matches.value_of("key").unwrap_or("").to_string(),
        }
    }
}
