//! Types used to move data through the API.

use super::state::{Message};

use time::OffsetDateTime;
use serde::Deserialize;

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

#[cfg(test)]
mod tests {
    use super::*;

    mod incoming_message {
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
    }
}
