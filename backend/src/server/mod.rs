use std::{net::SocketAddr, sync::Arc, sync::Mutex};
use axum::{extract::{ws::Message, ConnectInfo, State, WebSocketUpgrade}, response::Response, routing::get, Router};
use futures::{Sink, SinkExt, Stream, StreamExt};
use state::AppState;
use tokio::net::TcpListener;
use crate::realtime::process_messsage;

#[cfg(test)]
mod tests;
mod realtime;
mod state;

#[tokio::main]
pub async fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    let state: AppState = AppState::new();
    let app: Router = Router::new().route("/ws", get(handshake)).with_state(Arc::new(Mutex::new(state)));
    tracing_subscriber::fmt::init();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handshake(
    ws: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<AppState>>>
) -> Response {
    ws.on_upgrade(move |socket| {
        let (sender, receiver) = socket.split();
        handle_socket(sender, receiver, address, state)
    })
}

async fn handle_socket<
    W: Sink<Message> + Unpin,
    R: Stream<Item = Result<Message, axum::Error>> + Unpin>(
    mut sender: W, mut receiver: R,
    socket_address: SocketAddr,
    state: Arc<Mutex<AppState>>
) {
    while let Some(Ok(Message::Text(message))) = receiver.next().await {
        let response: serde_json::Value = match serde_json::from_str::<realtime::Message>(&message) {
            Err(_) => todo!(),
            Ok(message) => process_messsage(message, socket_address, state.clone()).await.unwrap()
        };
        if sender.send(Message::Text(response.to_string())).await.is_err() { break; }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}