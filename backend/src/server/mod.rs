use std::{net::SocketAddr, sync::Arc};
use axum::{extract::{ws::Message, ConnectInfo, State, WebSocketUpgrade}, response::Response, routing::get, Router};
use futures::{Sink, SinkExt, Stream, StreamExt};
use realtime::process_messsage;
use tokio::{net::TcpListener, sync::Mutex};
use crate::realtime::AppState;

mod realtime;

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

async fn handle_socket<W, R>(
    mut sender: W, mut receiver: R,
    socket_address: SocketAddr,
    state: Arc<Mutex<AppState>>
) where
    W: Sink<Message> + Unpin,
    R: Stream<Item = Result<Message, axum::Error>> + Unpin
{
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(message) = message {
            let response =  process_messsage(message, socket_address, state.clone()).await.unwrap();
            if sender.send(Message::Text(response.to_string())).await.is_err() { break; }
        }
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
