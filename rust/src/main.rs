use axum::extract;
use axum::extract::ws::{self, WebSocket, WebSocketUpgrade};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use axum::Router;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    // let listener = TcpListener::bind("localhost:8080").await?;
    println!("[INFO] server listening: {:?}", listener);

    let state = Arc::new(State::new());

    let _ = tokio::spawn({
        let server = state.clone();
        async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            server.clear().await;
        }
    });

    let app = Router::new()
        .route("/api/newuser", routing::get(newuser_handler))
        .route("/api/room", routing::any(room_handler))
        .with_state(state);

    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct NewuserQuery {
    key: String,
}

async fn newuser_handler(
    extract::State(state): extract::State<Arc<State>>,
    extract::Query(NewuserQuery { key }): extract::Query<NewuserQuery>,
) -> (StatusCode, HeaderMap) {
    println!("[INFO] /api/newuser: key == {key}");
    if key.len() != 5 || key.chars().any(|c| !c.is_ascii_lowercase()) {
        return (StatusCode::NOT_FOUND, HeaderMap::new());
    }

    let room = state.get_room(&key).await;
    let (id, _) = room.new_user().await;
    println!("[INFO] state == {:?}", state);
    (
        StatusCode::SEE_OTHER,
        HeaderMap::from_iter([(
            header::LOCATION,
            HeaderValue::from_str(&format!("/room/{key}/{id}")).unwrap(),
        )]),
    )
}

#[derive(Deserialize)]
struct RoomQuery {
    key: String,
    id: u32,
}

async fn room_handler(
    extract::State(state): extract::State<Arc<State>>,
    extract::Query(RoomQuery { key, id }): extract::Query<RoomQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    println!("[INFO] /api/room: key == {key}, id == {id}");
    if key.len() != 5 || key.chars().any(|c| !c.is_ascii_lowercase()) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let room = state.get_room(&key).await;
    let Some(user) = room.users.lock().await.get(&id).map(|user| user.clone()) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    println!("[INFO] state == {:?}", state);

    ws.on_upgrade(move |socket| room_socket_handler(room, id, user, socket))
}
async fn room_socket_handler(room: Arc<Room>, user_id: u32, user: Arc<User>, socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    let _ = sender
        .send(extract::ws::Message::Text("HI from server".into()))
        .await;

    let _ = tokio::join!(
        {
            let room = room.clone();
            async move {
                let mut len = 0;
                loop {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    let messages = room.messages.lock().await;
                    if messages.len() > len {
                        if sender
                            .send(extract::ws::Message::Text(
                                messages.get(len).unwrap().into(),
                            ))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        len += 1;
                    }
                }
            }
        },
        async move {
            while let Some(Ok(message)) = receiver.next().await {
                match message {
                    ws::Message::Text(message) => {
                        room.messages.lock().await.push(match &user.name {
                            Some(name) => format!("{message} by {name} <{user_id}>"),
                            None => format!("{message} by <{user_id}>"),
                        })
                    }
                    _ => println!("[INFO] got a message from websocket: {:?}", message),
                }
            }
        }
    );
}

#[derive(Debug)]
struct State {
    rooms: Mutex<HashMap<String, Arc<Room>>>,
}

impl State {
    fn new() -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
        }
    }
    async fn clear(&self) {
        let now = Instant::now();
        self.rooms.lock().await.retain(|_, room| {
            let mut users = room.users.blocking_lock();
            users.retain(|_, user| {
                !matches!(user.connection_state, ConnectionState::LastTime(instant) if now.duration_since(instant) > Duration::from_secs(30))
            });
            users.is_empty()
        });
    }
    async fn get_room(&self, key: &str) -> Arc<Room> {
        let mut rooms = self.rooms.lock().await;
        match rooms.get(key) {
            Some(room) => room.clone(),
            None => {
                let room = Arc::new(Room::new());
                let _ = rooms.insert(key.to_owned(), room.clone());
                room
            }
        }
    }
}

#[derive(Debug)]
struct Room {
    users: Mutex<HashMap<u32, Arc<User>>>,
    messages: Mutex<Vec<String>>,
}

impl Room {
    fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            messages: Mutex::new(Vec::new()),
        }
    }
    async fn new_user(&self) -> (u32, Arc<User>) {
        let mut users = self.users.lock().await;
        loop {
            let id = rand::random();
            if !users.contains_key(&id) {
                let user = Arc::new(User::new());
                users.insert(id, user.clone());
                break (id, user);
            }
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct User {
    name: Option<String>,
    connection_state: ConnectionState,
}

impl User {
    fn new() -> Self {
        Self {
            name: None,
            connection_state: ConnectionState::LastTime(Instant::now()),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
enum ConnectionState {
    Connected,
    LastTime(Instant),
}
