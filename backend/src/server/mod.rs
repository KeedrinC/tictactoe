use std::{net::SocketAddr, sync::Arc, sync::Mutex};
use axum::{
    extract::{connect_info::ConnectInfo, ws::Message, State, WebSocketUpgrade},
    response::Response,
    routing::any,
    Router
};
use futures::{Sink, SinkExt, Stream, StreamExt};
use messages::ClientMessage;
use serde_json::{json, Value};
use state::AppState;
use tokio::{net::TcpListener, sync::broadcast::Sender};

#[cfg(test)]
mod tests;
mod lobby;
mod messages;
mod session;
mod state;

#[tokio::main]
pub async fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    let state: AppState = AppState::new();
    let app: Router = Router::new()
        .route("/ws", any(handshake))
        .with_state(Arc::new(Mutex::new(state)));
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handshake(
    ws: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<AppState>>>
) -> Response {
    tracing::info!("new connection from {}:{}", address.ip(), address.port());
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
    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(message) => {
                let response: serde_json::Value = match serde_json::from_str::<messages::ClientMessage>(&message) {
                    Err(error) => { json!({"type": "Error", "data": error.to_string()}) },
                    Ok(message) =>
                        ClientMessage::process(message, socket_address, state.clone()).await
                            .map_or_else(|error| json!({"type": "Error", "data": error}), |message| message)
                };
                if sender.send(Message::Text(response.to_string().into())).await.is_err() { break; }
            },
            _ => {}
        };
        if let Some(channel) = get_lobby_channel(state.clone(), socket_address) {
            let mut rx = channel.subscribe();
            while let Ok(msg) = rx.recv().await {
                tracing::info!("{}", msg);
                if sender.send(Message::text(msg.to_string())).await.is_err() { break; }
            }
        }
    }
    
}

fn get_lobby_channel(state: Arc<Mutex<AppState>>, socket_address: SocketAddr) -> Option<Sender<Value>> {
    let state = state.lock().unwrap();
    let session = state.socket_session.get(&socket_address)?.lock().unwrap();
    let lobby = state.session_lobby.get(&session.access_token)?.lock().unwrap();
    let channel = state.lobby_channel.get(&lobby.code.clone())?;
    Some(channel.0.clone())
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