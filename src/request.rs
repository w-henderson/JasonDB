#![allow(dead_code)]
use crate::Database;
use parking_lot::RwLock;
use serde_json::Value;
use std::sync::Arc;

#[derive(Eq, PartialEq, Debug)]
pub enum Request<'a> {
    Create {
        collection: &'a str,
    },
    Get {
        collection: &'a str,
        document: &'a str,
    },
    Set {
        collection: &'a str,
        document: &'a str,
        value: String,
    },
    List {
        collection: &'a str,
        condition: Option<Condition>,
    },
    Delete {
        collection: &'a str,
    },
    Invalid {
        error: &'a str,
    },
}

/// Represents a response from the server.
#[derive(Eq, PartialEq, Debug)]
pub enum Response {
    Success { data: Option<String> },
    Error { message: String },
}

impl Response {
    /// Create a successful response object.
    pub fn success(data: Option<String>) -> Self {
        Self::Success { data }
    }

    /// Create an error response object.
    pub fn error(message: &str) -> Self {
        Self::Error {
            message: message.to_string(),
        }
    }

    /// Convert the response into a JSON string.
    pub fn to_json(&self) -> String {
        match self {
            Response::Success { data } => {
                if let Some(data) = data {
                    format!("{{\"status\": \"success\", \"data\": {}}}", data)
                } else {
                    r#"{"status": "success"}"#.to_string()
                }
            }
            Response::Error { message } => {
                format!(
                    "{{\"status\": \"error\", \"message\": \"{}\"}}",
                    message.replace("\"", "\\\"")
                )
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Condition {
    Eq { key: String, value: String },
    Gt { key: String, value: String },
    Lt { key: String, value: String },
}

impl Condition {
    pub fn parse(string: &[&str]) -> Option<Self> {
        if string.len() != 3 {
            None
        } else {
            match string[1] {
                "EQ" => Some(Self::Eq {
                    key: string[0].to_string(),
                    value: string[2].to_string(),
                }),
                "GT" => Some(Self::Gt {
                    key: string[0].to_string(),
                    value: string[2].to_string(),
                }),
                "LT" => Some(Self::Lt {
                    key: string[0].to_string(),
                    value: string[2].to_string(),
                }),
                _ => None,
            }
        }
    }
}

/// Parses a request string into a `Request` object.
/// For example, the request string "GET CoolTomato FROM users" would be parsed into:
/// ```rs
/// Request::Get {
///     collection: "users",
///     document: "CoolTomato"
/// }
/// ```
pub fn parse(string: &str) -> Request {
    let parsed_string: Vec<&str> = string.split_ascii_whitespace().collect();
    let len = parsed_string.len();
    if len < 2 {
        return Request::Invalid {
            error: "Unknown command",
        };
    };

    match parsed_string[0] {
        "CREATE" => {
            if len == 2 {
                Request::Create {
                    collection: parsed_string[1],
                }
            } else {
                Request::Invalid {
                    error: "CREATE command is formatted as 'CREATE <collection>'",
                }
            }
        }

        "GET" => {
            if len == 4 && parsed_string[2] == "FROM" {
                Request::Get {
                    collection: parsed_string[3],
                    document: parsed_string[1],
                }
            } else {
                Request::Invalid {
                    error: "GET command is formatted as 'GET <document> FROM <collection>'",
                }
            }
        }

        "SET" => {
            if len >= 6 && parsed_string[2] == "FROM" && parsed_string[4] == "TO" {
                Request::Set {
                    collection: parsed_string[3],
                    document: parsed_string[1],
                    value: parsed_string[5..].join(" "),
                }
            } else {
                Request::Invalid {
                    error:
                        "SET command is formatted as 'SET <document> FROM <collection> TO <value>'",
                }
            }
        }

        "LIST" => {
            if len == 2 {
                Request::List {
                    collection: parsed_string[1],
                    condition: None,
                }
            } else if len >= 4 && parsed_string[2] == "WHERE" {
                let parsed_condition = Condition::parse(&parsed_string[3..]);
                if parsed_condition.is_some() {
                    Request::List {
                        collection: parsed_string[1],
                        condition: parsed_condition,
                    }
                } else {
                    Request::Invalid {
                        error: "Condition keywords are EQ, LT, or GT",
                    }
                }
            } else {
                Request::Invalid {
                    error: "LIST command is formatted as 'LIST <collection> [WHERE <condition>]",
                }
            }
        }

        "DELETE" => {
            if len == 2 {
                Request::Delete {
                    collection: parsed_string[1],
                }
            } else {
                Request::Invalid {
                    error: "DELETE command is formatted as 'DELETE <collection>'",
                }
            }
        }

        _ => Request::Invalid {
            error: "Unknown command",
        },
    }
}

/// Executes a request object and returns a `Response`.
/// This is either `Response::Success` or `Response::Error`.
pub fn execute(request: Request, db_ref: &Arc<RwLock<Database>>) -> Response {
    match request {
        Request::Create { collection } => {
            let mut db = db_ref.write();
            let result = (*db).create_collection(collection);
            if result.is_ok() {
                db.increment_writes();
                Response::success(None)
            } else {
                Response::error("Collection already exists")
            }
        }

        Request::Get {
            collection,
            document,
        } => {
            let db = db_ref.read();
            let collection_option = (*db).collection(collection);
            if let Some(coll) = collection_option {
                let document_option = coll.get(document);
                if let Some(doc) = document_option {
                    Response::success(Some(doc.json.clone()))
                } else {
                    Response::error("Document not found")
                }
            } else {
                Response::error("Collection not found")
            }
        }

        Request::Set {
            collection,
            document,
            value,
        } => {
            let mut db = db_ref.write();
            let collection_option = (*db).collection_mut(collection);
            if let Some(coll) = collection_option {
                if coll.set(document, value) {
                    db.increment_writes();
                    Response::success(None)
                } else {
                    Response::error("Invalid JSON")
                }
            } else {
                Response::error("Collection not found")
            }
        }

        Request::List {
            collection,
            condition,
        } => {
            let db = db_ref.read();
            let collection_option = (*db).collection(collection);
            if let Some(coll) = collection_option {
                if coll.list().len() == 0 {
                    return Response::success(Some("[]".to_string()));
                };

                if let Some(condition) = condition {
                    let mut json = coll
                        .list()
                        .iter()
                        .filter(|item| {
                            let parsed_item: Value = serde_json::from_str(&item.json).unwrap();
                            match &condition {
                                Condition::Eq { key, value } => {
                                    if let Some(actual_value) = parsed_item.get(&key) {
                                        if let Some(string_value) = actual_value.as_str() {
                                            string_value == value
                                        } else if let Some(numeric_value) = actual_value.as_f64() {
                                            if let Ok(target_value) = value.parse::<f64>() {
                                                target_value == numeric_value
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }
                                Condition::Gt { key, value } => {
                                    if let Some(actual_value) = parsed_item.get(&key) {
                                        if let Some(numeric_value) = actual_value.as_f64() {
                                            if let Ok(specified_value) = value.parse::<f64>() {
                                                numeric_value > specified_value
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }
                                Condition::Lt { key, value } => {
                                    if let Some(actual_value) = parsed_item.get(&key) {
                                        if let Some(numeric_value) = actual_value.as_f64() {
                                            if let Ok(specified_value) = value.parse::<f64>() {
                                                numeric_value < specified_value
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }
                            }
                        })
                        .fold("[".to_string(), |acc, doc| {
                            acc + "{\"id\": \"" + &doc.id + "\", \"data\": " + &doc.json + "}, "
                        });

                    if json == "[" {
                        return Response::success(Some("[]".to_string()));
                    };
                    json.drain(json.len() - 2..);
                    json += "]";

                    Response::success(Some(json))
                } else {
                    let mut json = coll.list().iter().fold("[".to_string(), |acc, doc| {
                        acc + "{\"id\": \"" + &doc.id + "\", \"data\": " + &doc.json + "}, "
                    });
                    json.drain(json.len() - 2..);
                    json += "]";

                    Response::success(Some(json))
                }
            } else {
                Response::error("Collection not found")
            }
        }

        Request::Delete { collection } => {
            let mut db = db_ref.write();
            let result = (*db).delete_collection(collection);
            if result.is_ok() {
                db.increment_writes();
                Response::success(None)
            } else {
                Response::error("Collection not found")
            }
        }

        Request::Invalid { error } => Response::error(error),
    }
}
