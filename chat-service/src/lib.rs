use std::future::Future;
use std::ops::DerefMut;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing, Form, Json, Router};
use chatroom::{Chatroom, IncomingMessage, Message};
use tokio::sync::{Mutex, MutexGuard};
use tower_http::limit::RequestBodyLimitLayer;

#[derive(Debug, Clone)]
pub struct ChatServiceState {
    chatroom: Arc<Mutex<Chatroom>>,
}

impl ChatServiceState {
    pub fn with_room(chatroom: Chatroom) -> Self {
        Self {
            chatroom: Arc::new(Mutex::new(chatroom)),
        }
    }

    pub fn chatroom_lock(&self) -> impl Future<Output = MutexGuard<Chatroom>> {
        self.chatroom.lock()
    }
}

impl Default for ChatServiceState {
    fn default() -> Self {
        Self::with_room(Chatroom::default())
    }
}

pub fn router() -> Router<ChatServiceState> {
    Router::new()
        .route("/post/form", routing::post(post_form))
        .route("/post/json", routing::post(post_json))
        .layer(RequestBodyLimitLayer::new(1024 * 4))
        .route("/get/all", routing::get(get_all))
}

async fn get_all(State(state): State<ChatServiceState>) -> (StatusCode, Json<Vec<Arc<Message>>>) {
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

async fn post(state: ChatServiceState, message: IncomingMessage) -> (StatusCode, String) {
    if message.is_valid() {
        let complete_message: Message = message.into();
        let mut lock = state.chatroom_lock().await;
        let index_placed = lock.deref_mut().add(complete_message);
        drop(lock);
        (StatusCode::CREATED, index_placed.to_string())
    } else {
        (StatusCode::BAD_REQUEST, "".into())
    }
}

async fn post_form(
    State(state): State<ChatServiceState>,
    Form(message): Form<IncomingMessage>,
) -> (StatusCode, String) {
    post(state, message).await
}

async fn post_json(
    State(state): State<ChatServiceState>,
    Json(message): Json<IncomingMessage>,
) -> (StatusCode, String) {
    post(state, message).await
}
