use std::sync::Arc;

use axum::{extract::{Request, State}, http::StatusCode, routing::get, Json, Router};
use chatroom::{Chatroom, Message};
use tokio::sync::Mutex;
use tower::Service;

#[derive(Debug, Clone)]
pub struct ChatService {
    chatroom: Arc<Mutex<Chatroom>>,
}

impl ChatService {
    pub fn with_room(chatroom: Arc<Mutex<Chatroom>>) -> Self{
        Self {
            chatroom,
        }
    }

    pub fn router(&self) -> Router {
        Router::new()
        .route("/get/all", get(get_all))
        .with_state(self)
    }
}

async fn get_all(State(state): State<&ChatService>) -> (StatusCode, Json<Vec<Arc<Message>>>) {
    let lock = state.chatroom.lock().await;
    match lock.try_get_range(..) {
        Some(value) => (StatusCode::CREATED, Json::from(value)),
        None => (StatusCode::BAD_REQUEST, Json::from(vec![])),
    }
}
