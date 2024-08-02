use std::future::Future;
use std::sync::Arc;

use axum::extract::State;
use axum::{http::StatusCode, routing, Json, Router};
use chatroom::{Chatroom, Message};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct ChatService {
    chatroom: Arc<Mutex<Chatroom>>,
}

impl ChatService {
    pub fn with_room(chatroom: Arc<Mutex<Chatroom>>) -> Self {
        Self { chatroom }
    }

    pub fn router(&self) -> Router<Self> {
        Router::new()
            .route("/get/all", routing::get(get_all))
            .with_state(self.clone())
    }

    pub fn chatroom_lock(&self) -> impl Future<Output = MutexGuard<Chatroom>> {
        self.chatroom.lock()
    }
}

async fn get_all(State(state): State<ChatService>) -> (StatusCode, Json<Vec<Arc<Message>>>) {
    let chat_state: Option<Vec<Arc<Message>>> = {
        // Isolate the Mutex from any function errors and/or panics
        let lock = state.chatroom_lock().await;
        lock.try_get_range(..)
    };

    match chat_state {
        Some(messages) => (StatusCode::OK, Json(messages)),
        None => (StatusCode::SERVICE_UNAVAILABLE, Json([].into())),
    }
}
