use std::ops::Add;
use std::{collections::HashMap, sync::Arc};
use std::time;
use rand::RngExt;

use axum::{Router, extract::{Path, State}, http::StatusCode, routing::{delete, get, put}};
use tokio::sync::RwLock;

#[derive(Clone)]
struct ValueStore {
    expiry: time::Instant,
    value: String,
}

#[tokio::main]
async fn main() {
    let store: Arc<RwLock<HashMap<String, ValueStore>>> = Arc::new(RwLock::new(HashMap::new()));
    let store_clone = store.clone();

    let app = Router::new()
        .route("/keys",        put(set_key))
        .route("/keys/{name}", get(get_key))
        .route("/keys/{name}", delete(delete_key))
        .with_state(store);


    let _thread = std::thread::spawn(async move || {
        loop {
            std::thread::sleep(time::Duration::from_mins(1));
            let now = time::Instant::now();
            let store = store_clone.write().await;
            let _ = store.iter()
                .filter(|(_, v)| {
                    now < v.expiry
                });
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("keyserver listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_key(Path(name): Path<String>, State(store): State<Arc<RwLock<HashMap<String, ValueStore>>>>) -> Result<String, StatusCode> {
    let store = store.read().await;
    let value = store.get(&name).ok_or(StatusCode::NOT_FOUND)?;

    let now = time::Instant::now();
    if now > value.expiry {
        return Err(StatusCode::NOT_FOUND)
    }

    Ok(value.value.clone())
}

async fn set_key(State(store): State<Arc<RwLock<HashMap<String, ValueStore>>>>, body: String) -> Result<String, StatusCode> {
    let mut store = store.write().await;

    let mut key = generate_pin();
    while store.contains_key(&key) {
        key = generate_pin();
    }

    let value = ValueStore {
        expiry: time::Instant::now().add(time::Duration::from_mins(5)),
        value: body.trim().to_string()
    };

    store.insert(key.clone(), value);
    Ok(key)
}

async fn delete_key(Path(name): Path<String>, State(store): State<Arc<RwLock<HashMap<String, ValueStore>>>>) -> StatusCode {
    let mut store = store.write().await;
    match store.remove(&name) {
        Some(_) => StatusCode::NO_CONTENT,
        None => StatusCode::NOT_FOUND,
    }
}


const KEY_CHARS: &[u8] = b"0123456789ABCDEF";

pub fn generate_pin() -> String {
    let mut rng = rand::rng();
    (0..6)
        .map(|_| KEY_CHARS[rng.random_range(0..KEY_CHARS.len())] as char)
        .collect()
}