use axum::extract;
use axum::extract::ws::{self, WebSocket, WebSocketUpgrade};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use axum::Router;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("[INFO] server listening: {:?}", listener);

    let state = Arc::new(State::new());

    let _ = tokio::spawn({
        let server = state.clone();
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                println!("[INFO] start clear");
                server.clear().await;
                println!("[INFO] end clear");
            }
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
async fn room_socket_handler(
    room: Arc<Room>,
    user_id: u32,
    user: Arc<Mutex<User>>,
    socket: WebSocket,
) {
    user.lock().await.connection_state = ConnectionState::Connected;
    let (mut sender, mut receiver) = socket.split();

    if let Some(name) = &user.lock().await.name {
        let _ = sender
            .send(extract::ws::Message::Text(
                serde_json::to_string(&SocketUpdateMessage::YourName(name.clone()))
                    .unwrap()
                    .into(),
            ))
            .await;
    }
    //let _ = sender
    //    .send(ws::Message::Text("HI from server".into()))
    //    .await;

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
                                serde_json::to_string(&SocketUpdateMessage::NewMessage(
                                    messages.get(len).unwrap().clone(),
                                ))
                                .unwrap()
                                .into(),
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
        {
            let user = user.clone();
            async move {
                while let Some(Ok(message)) = receiver.next().await {
                    match message {
                        ws::Message::Text(message) => {
                            // TODO: remove unwrap
                            match serde_json::from_str::<SocketMessage>(&message).unwrap() {
                                SocketMessage::NewMessage(message) => {
                                    room.messages.lock().await.push(Message {
                                        content: message,
                                        user_id,
                                        user_name: user.lock().await.name.clone(),
                                    })
                                }
                                SocketMessage::UpdateName(name) => {
                                    user.lock().await.name = Some(name)
                                }
                            }
                        }
                        _ => println!("[INFO] got a message from websocket: {:?}", message),
                    }
                }
            }
        }
    );

    user.lock().await.connection_state = ConnectionState::LastTime(Instant::now());
    println!("[INFO] a socket end and close, {:?}", user);
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
enum SocketMessage {
    #[serde(rename = "new_message")]
    NewMessage(String),
    #[serde(rename = "update_name")]
    UpdateName(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
enum SocketUpdateMessage {
    #[serde(rename = "new_message")]
    NewMessage(Message),
    #[serde(rename = "your_name")]
    YourName(String),
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
        println!("[TICK] tick a");
        let now = Instant::now();
        self.rooms.lock().await.retain(|_, room| {
            println!("[TICK] tick b");
            let mut users = tokio::task::block_in_place(|| room.users.blocking_lock());
            println!("[TICK] tick b.2");
            users.retain(|_, user| !matches!(
                tokio::task::block_in_place(|| user.blocking_lock()).connection_state,
                ConnectionState::LastTime(instant) if now.duration_since(instant) > Duration::from_secs(30)
            ));
            println!("[TICK] tick c");
            !users.is_empty()
        });
        println!("[INFO] a clear, rooms after clear: {:?}", self.rooms);
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
    users: Mutex<HashMap<u32, Arc<Mutex<User>>>>,
    messages: Mutex<Vec<Message>>,
}

impl Room {
    fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            messages: Mutex::new(Vec::new()),
        }
    }
    async fn new_user(&self) -> (u32, Arc<Mutex<User>>) {
        let mut users = self.users.lock().await;
        loop {
            let id = rand::random();
            if !users.contains_key(&id) {
                let user = Arc::new(Mutex::new(User::new()));
                users.insert(id, user.clone());
                break (id, user);
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message {
    content: String,
    user_id: u32,
    user_name: Option<String>,
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
