use std::cmp::min;
use std::ops::{Bound, RangeBounds};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use time::serde::iso8601;
use time::OffsetDateTime;

/// A [`Mutex`] containing a [`Chatroom`]
pub type API = Mutex<Chatroom>;

/// A chat message.
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    /// When the message was created.
    #[serde(with = "iso8601")]
    pub time: OffsetDateTime,
    /// Who the message came from.
    pub sender: String,
    /// The actual message.
    pub body: String,
}

/// A room that contains [`Message`]s.
///
/// To be held in a [`Mutex`].
#[derive(Debug)]
pub struct Chatroom {
    /// The message queue.
    messages: Vec<Arc<Message>>,
}

impl Chatroom {
    pub fn new() -> Arc<Chatroom> {
        let messages = Vec::new();

        Arc::new(Chatroom { messages })
    }

    pub fn add(&mut self, message: Message) -> usize {
        self.messages.push(Arc::new(message));
        self.messages.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<Arc<Message>> {
        self.messages.get(index).cloned()
    }

    pub fn try_get_range(&self, index: impl RangeBounds<usize>) -> Option<Vec<Arc<Message>>> {
        // start is inclusive
        let start: usize = match index.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1, // ok to panic
            Bound::Unbounded => 0,
        };
        // end is exclusive
        let end: usize = min(
            self.messages.len(),
            match index.end_bound() {
                Bound::Included(&i) => i + 1, // ok to panic
                Bound::Excluded(&i) => i,
                Bound::Unbounded => usize::MAX, // equivalent to messages.len() because of the surrounding min()
            },
        ); // the min function makes sure that end <= messages.len()
        if start >= end {
            None
        } else {
            // We know that 0 <= start < end <= messages.len(), so we know start is in-bounds.
            Some(Vec::from_iter(
                self.messages[start..end].iter().cloned()
            ))
        }
    }
}
