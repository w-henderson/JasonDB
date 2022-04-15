//! Provides replication functionality through traits.

use crate::error::JasonError;
use crate::sources::Source;
use crate::Database;

use humphrey_json::prelude::*;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{spawn, JoinHandle};

/// Represents a replica of a database.
///
/// The type parameter `T` represents the datatype of the database. However, since the replica is not necessarily
///   using Rust types, the replica handles only the serialized JSON version of the value.
pub trait Replica<T>: Send + 'static {
    /// Replicate the change to the replica.
    ///
    /// The value is passed as the JSON representation of the value.
    fn set(&mut self, key: &str, value: &str) -> Result<(), JasonError>;
}

/// Manages replication to a replica.
pub(crate) enum Replicator<T> {
    /// A synchronous replica.
    Sync(Box<dyn Replica<T> + Send>),

    /// An asynchronous replica which manages a thread and a channel for communication.
    Async {
        /// The thread which manages the replica.
        thread: Option<JoinHandle<()>>,
        /// A sender to send messages to the thread.
        sender: Sender<ReplicationMessage>,
    },
}

/// Represents a message to be sent to an asynchronous replica management thread.
pub(crate) enum ReplicationMessage {
    /// Indicates that the thread should replicate this write.
    Replicate(String, String),
    /// Indicates that the thread should shut down.
    Shutdown,
}

impl<T> Replicator<T>
where
    T: 'static,
{
    /// Creates a new synchronous replicator.
    pub fn new<R>(replica: R) -> Self
    where
        R: Replica<T>,
    {
        Self::Sync(Box::new(replica))
    }

    /// Creates a new asynchronous replicator.
    pub fn new_async<R>(mut replica: R) -> Self
    where
        R: Replica<T>,
    {
        let (tx, rx): (Sender<ReplicationMessage>, Receiver<ReplicationMessage>) = channel();

        let handle = spawn(move || {
            for msg in rx {
                match msg {
                    ReplicationMessage::Replicate(key, value) => {
                        replica.set(&key, &value).unwrap();
                    }
                    ReplicationMessage::Shutdown => {
                        break;
                    }
                }
            }
        });

        Self::Async {
            thread: Some(handle),
            sender: tx,
        }
    }

    /// Sets the key to the given value in the replica.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), JasonError> {
        match self {
            Self::Sync(replica) => replica.set(key, value),
            Self::Async { sender, .. } => {
                let msg = ReplicationMessage::Replicate(key.to_string(), value.to_string());

                sender.send(msg).map_err(|_| JasonError::ReplicaError)?;

                Ok(())
            }
        }
    }
}

impl<T> Drop for Replicator<T> {
    fn drop(&mut self) {
        match self {
            Self::Sync(_) => (),
            Self::Async { thread, sender } => {
                sender.send(ReplicationMessage::Shutdown).unwrap();

                if let Some(thread) = thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
}

impl<T, S> Replica<T> for Database<T, S>
where
    T: IntoJson + FromJson + Send + 'static,
    S: Source + Send + 'static,
{
    fn set(&mut self, key: &str, value: &str) -> Result<(), JasonError> {
        self.set_raw(key, value.as_bytes())
    }
}
