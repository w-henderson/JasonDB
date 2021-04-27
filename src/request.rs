#![allow(dead_code)]
use crate::Database;
use parking_lot::RwLock;
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
        condition: Option<Condition<'a>>,
    },
    Delete {
        collection: &'a str,
    },
    Invalid {
        error: &'a str,
    },
}

#[derive(Eq, PartialEq, Debug)]
pub enum Condition<'a> {
    Eq { key: &'a str, value: &'a str },
    Gt { key: &'a str, value: &'a str },
    Lt { key: &'a str, value: &'a str },
}

impl Condition<'_> {
    pub fn parse(_string: &[&str]) -> Option<Self> {
        // TODO: IMPLEMENT
        Some(Self::Eq { key: "", value: "" })
    }
}

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

pub fn execute(request: Request, db_ref: &Arc<RwLock<Database>>) -> String {
    match request {
        Request::Create { collection } => {
            let mut db = db_ref.write();
            let result = (*db).create_collection(collection);
            if result.is_ok() {
                r#"{"status": "success"}"#.to_string()
            } else {
                r#"{"status": "error", "message": "Collection already exists"}"#.to_string()
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
                    format!("{{\"status\": \"success\", \"data\": {}}}", doc.json)
                } else {
                    r#"{"status": "error", "message": "Document not found"}"#.to_string()
                }
            } else {
                r#"{"status": "error", "message": "Collection not found"}"#.to_string()
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
                coll.set(document, value);
                r#"{"status": "success"}"#.to_string()
            } else {
                r#"{"status": "error", "message": "Collection not found"}"#.to_string()
            }
        }

        Request::List {
            collection,
            .. // TODO: IMPLEMENT CONDITION
        } => {
            let db = db_ref.read();
            let collection_option = (*db).collection(collection);
            if let Some(coll) = collection_option {
                let json = coll
                    .list()
                    .iter()
                    .fold("[".to_string(), |acc, doc| acc + &doc.json + ", ")
                    + "]";
                format!("{{\"status\": \"success\", data: {}}}", json)
            } else {
                r#"{"status": "error", "message": "Collection not found"}"#.to_string()
            }
        }

        Request::Delete { collection } => {
            let mut db = db_ref.write();
            let result = (*db).delete_collection(collection);
            if result.is_ok() {
                r#"{"status": "success"}"#.to_string()
            } else {
                r#"{"status": "error", "message": "Collection not found"}"#.to_string()
            }
        }

        Request::Invalid { error } => {
            format!("{{\"status\": \"error\", \"message\": \"{}\"}}", error)
        }
    }
}
