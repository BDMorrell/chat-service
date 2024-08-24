use std::cmp::min;
use std::ops::{Bound, RangeBounds};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use time::serde::iso8601;
use time::OffsetDateTime;

/// A chat message.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    /// When the message was created.
    #[serde(with = "iso8601")]
    pub time: OffsetDateTime,
    /// Who the message came from.
    pub sender: String,
    /// The actual message.
    pub body: String,
}

/// A chat message to be added to a [`Chatroom`].
///
/// SECURITY TODO: Currently, there is no validation ofther than the message
/// seems to be utf-8. This implementation currrently does not validate,
/// normalize, nor remove control characters.
#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    /// Who the message came from.
    pub sender: String,
    /// The actual message.
    pub body: String,
}

impl From<IncomingMessage> for Message {
    fn from(value: IncomingMessage) -> Self {
        Self {
            time: OffsetDateTime::now_utc(),
            sender: value.sender,
            body: value.body,
        }
    }
}

impl IncomingMessage {
    /// Make sure a message has nonempty fields.
    ///
    /// SECURITY TODO: Currently, there is no validation ofther than the message
    /// seems to be utf-8. This implementation currrently does not validate,
    /// normalize, nor remove control characters.
    pub fn is_valid(&self) -> bool {
        !self.sender.is_empty() && !self.body.is_empty()
    }
}

/// A room that contains [`Message`]s.
///
/// To be held in a [`Mutex`].
#[derive(Debug, Default)]
pub struct Chatroom {
    /// The message queue.
    messages: Vec<Arc<Message>>,
}

impl Chatroom {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a message to the chatroom.
    ///
    /// Returns the index the message was placed into.
    pub fn add(&mut self, message: Message) -> usize {
        self.messages.push(Arc::new(message));
        self.messages.len() - 1
    }

    /// Retrives a message from the chatroom.
    /// 
    /// Returns [`None`] when given an index that is out of bounds.
    pub fn get(&self, index: usize) -> Option<Arc<Message>> {
        self.messages.get(index).cloned()
    }

    /// Tries to get the range specified.
    /// 
    /// [`None`] is returned if the range's `start` > `end`, or if `start`
    /// index is not a vliad index (i.e. too large). If `end` is too large, it
    /// is treated as though `end` is [`Bound::Unbounded`], which just means
    /// that it grabs all messages until the end.
    pub fn try_get_range(&self, index: impl RangeBounds<usize>) -> Option<Vec<Arc<Message>>> {
        // start and end are both inclusive
        let start: usize = match index.start_bound() {
            Bound::Included(&i) => i,
            // I'm not aware of a range that has an exclusive start, so this
            // next line shouldn't be able to run, which means it shouldn't be
            // able to panic.
            Bound::Excluded(&i) => i + 1, // Code included for completeness.
            Bound::Unbounded => 0,
        };
        // end is inclusive
        let end: usize = min(
            self.messages.len() - 1,
            match index.end_bound() {
                Bound::Included(&i) => i,
                Bound::Excluded(&i) => i.saturating_sub(1), // in case the end is 0
                Bound::Unbounded => usize::MAX, // equivalent to messages.len() because of the surrounding min()
            },
        ); // the min function makes sure that end <= messages.len()
        if start > end {
            None
        } else {
            // We know that 0 <= start < end <= messages.len(), so we know start is in-bounds.
            Some(Vec::from_iter(self.messages[start..=end].iter().cloned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_incoming() {
        let incoming = IncomingMessage {
            sender: "".into(),
            body: "".into(),
        };

        assert_eq!(false, incoming.is_valid());
    }
    #[test]
    fn empty_incoming_sender() {
        let incoming = IncomingMessage {
            sender: "".into(),
            body: "Is the server working?".into(),
        };

        assert_eq!(false, incoming.is_valid());
    }
    #[test]
    fn empty_incoming_body() {
        let incoming = IncomingMessage {
            sender: "Tester".into(),
            body: "".into(),
        };

        assert_eq!(false, incoming.is_valid());
    }
    #[test]
    fn nonempty_incoming() {
        let incoming = IncomingMessage {
            sender: "Tester".into(),
            body: "Is the server working?".into(),
        };

        assert!(incoming.is_valid());
    }

    fn get_demo_messages() -> [Message; 2] {
        [
            Message {
                time: OffsetDateTime::UNIX_EPOCH,
                sender: "Unix".into(),
                body: "The time is 0.".into(),
            },
            Message {
                time: OffsetDateTime::UNIX_EPOCH,
                sender: "Unixish".into(),
                body: "The timer is unset.".into(),
            },
        ]
    }

    #[test]
    fn add_messages() {
        let [msg0, msg1] = get_demo_messages();
        let mut room = Chatroom::new();
        assert_eq!(0, room.add(msg0.clone()));
        assert_eq!(1, room.add(msg1.clone()));

        assert_eq!(Some(Arc::new(msg0.clone())), room.get(0));
        assert_eq!(Some(Arc::new(msg1.clone())), room.get(1));
        assert_eq!(Some(Arc::new(msg0.clone())), room.get(0));
    }

    #[test]
    fn get_message_array_out_of_bounds() {
        let [msg0, msg1] = get_demo_messages();
        let mut room = Chatroom::new();
        room.add(msg0.clone());
        room.add(msg1.clone());

        assert_eq!(None, room.get(1024));
    }

    #[test]
    fn try_get_message0() {
        let [msg0, msg1] = get_demo_messages();
        let mut room = Chatroom::new();
        room.add(msg0.clone());
        room.add(msg1.clone());

        assert_eq!(Some([Arc::new(msg0.clone())].into()), room.try_get_range(0..=0)); // inclusive
        assert_eq!(Some([Arc::new(msg0)].into()), room.try_get_range(0..1)); // exclusive
    }

    #[test]
    fn try_get_message1() {
        let [msg0, msg1] = get_demo_messages();
        let mut room = Chatroom::new();
        room.add(msg0.clone());
        room.add(msg1.clone());

        assert_eq!(Some([Arc::new(msg1.clone())].into()), room.try_get_range(1..=1)); // inclusive
        assert_eq!(Some([Arc::new(msg1)].into()), room.try_get_range(1..2)); // exclusive
    }

    #[test]
    fn try_get_message_1_and_2() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(0..=1)); // inclusive
        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(0..2)); // exclusive
    }

    #[test]
    fn try_get_messages_with_abnormal_bounds() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(..=1));
        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(..));
        let [msg1, msg2] = get_demo_messages();
        assert_eq!(Some([Arc::new(msg1)].into()), room.try_get_range(..1));
        assert_eq!(Some([Arc::new(msg2)].into()), room.try_get_range(1..));
        assert_eq!(None, room.try_get_range(..0));
    }

    #[test]
    fn try_get_too_many_messages() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(0..=1024)); // inclusive
        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(0..1024)); // exclusive
    }

    #[test]
    fn try_get_messages_out_of_bounds() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(None, room.try_get_range(512..=1024)); // inclusive
        assert_eq!(None, room.try_get_range(512..1024)); // exclusive
    }

    #[test]
    fn try_get_reversed_ranges() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(None, room.try_get_range(1024..=5));
        assert_eq!(None, room.try_get_range(1024..6));
        assert_eq!(None, room.try_get_range(1..0));
        assert_eq!(None, room.try_get_range(1..1));
        assert_eq!(None, room.try_get_range(1..=0));
    }

    #[test]
    fn try_get_usize_extrema() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(Some(get_demo_messages().map(|m| Arc::new(m)).into()), room.try_get_range(usize::MIN..=usize::MAX));
    }

    #[test]
    fn try_get_usize_extrema_reversed() {
        let mut room = Chatroom::new();
        for msg in get_demo_messages().iter() {
            room.add(msg.clone());
        }

        assert_eq!(None, room.try_get_range(usize::MAX..=usize::MIN));
    }
}
