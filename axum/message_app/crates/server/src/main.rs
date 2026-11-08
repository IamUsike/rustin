use axum::{
    Router,
    extract::{
        ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers::UserAgent};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use std::{net::SocketAddr, ops::ControlFlow};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    //routes
    let app = Router::new()
        .route("/ws", any(ws_handler))
        //logging shi
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    //run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown Browser")
    };

    println!("{user_agent} at {addr} connected");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

//actual ws statemachine. 1 is spawned per connection
async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            //client disconnected
            return;
        };

        if process_message(&msg).is_break() {
            continue;
        }

        if socket.send(msg).await.is_err() {
            //client disconnected
            return;
        }

        println!("msg sent to {who}");
    }
}

fn process_message(msg: &Message) -> ControlFlow<(), ()> {
    //dont reply for ping, pong and close. cos tungstenite automatically does that
    match msg {
        Message::Text(_) => ControlFlow::Continue(()),
        Message::Binary(_) => ControlFlow::Continue(()),
        _ => ControlFlow::Break(()),
    }
}
