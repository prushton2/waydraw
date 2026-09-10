use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;
use iroh::{Endpoint, PublicKey, endpoint::{Connection, RecvStream, SendStream, presets}, protocol::{AcceptError, ProtocolHandler, Router}};

const ALPN: &[u8] = b"hello";

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum P2PError {
    ErrorDuring(String, Box<P2PError>),
    BindError(String),
    AcceptConnectionError(String),
    CreateConnectionError(String),
    InputError(String),
    ConnectionNotFound,
    PoisonedMutex,
    Timeout,
}

impl P2PError {
    pub fn during(reason: &str, error: P2PError) -> P2PError {
        P2PError::ErrorDuring(reason.to_string(), Box::new(error))
    }
}

impl From<P2PError> for String {
    fn from(value: P2PError) -> Self {
        match value {
            P2PError::ErrorDuring(r, e) => format!("{}: {}", r, String::from(*e)),
            P2PError::BindError(r) =>                     format!("{} (BindError)", r),
            P2PError::AcceptConnectionError(r) =>         format!("{} (AcceptConnectionError)", r),
            P2PError::CreateConnectionError(r) =>         format!("{} (CreateConnectionError)", r),
            P2PError::InputError(r) =>                    format!("{} (InputError)", r),
            P2PError::ConnectionNotFound =>                       String::from("(ConnectionNotFound)"),
            P2PError::PoisonedMutex =>                            String::from("(PoisonedMutex)"),
            P2PError::Timeout =>                                  String::from("(Timeout)"),
        }
    }
}

#[derive(Debug, Clone)]
struct Handler {
    conn: Arc<std::sync::Mutex<Option<Connection>>>
}

impl Handler {
    fn new() -> Self {
        Self {
            conn: Arc::new(std::sync::Mutex::new(None))
        }
    }
}

impl ProtocolHandler for Handler {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        *self.conn.lock().unwrap() = Some(conn);
        Ok(())
    }
}

pub struct P2P {
    handler: Box<Handler>,
    router: Router,
    send: Mutex<Option<SendStream>>,
    recv: Mutex<Option<RecvStream>>
}

impl P2P {
    pub async fn init() -> Result<(Self, PublicKey), P2PError> {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.map_err(|e| P2PError::during("Could not connect to hardware", P2PError::BindError(e.to_string())))?;
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();

        tokio::time::timeout(std::time::Duration::from_secs(5), ep.online()).await.map_err(|_| P2PError::during("Connection timed out communicating with Iroh relays. If this persists, ensure there is no middleman in your TLS connections or use a VPN.", P2PError::Timeout))?;

        let id = ep.id();

        Ok((Self {
            handler: handler,
            router: router,
            send: Mutex::new(None),
            recv: Mutex::new(None),
        }, id))
    }

    pub async fn await_connection(&mut self) -> Result<(), P2PError> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let mutex = self.handler.conn.lock().unwrap();
            if mutex.is_some() { break; }
        }

        let conn = self.handler.conn.lock().map_err(|_| P2PError::PoisonedMutex)?.as_mut().unwrap().clone();
        let (send, recv) = conn.accept_bi().await.map_err(|e| P2PError::during("Error accepting connection from client",P2PError::AcceptConnectionError(e.to_string())))?;

        self.send = Mutex::new(Some(send));
        self.recv = Mutex::new(Some(recv));

        let _ = self.read().await;
        Ok(())
    }

    pub async fn connect(id: PublicKey) -> Result<Self, P2PError> {
        let handler = Box::new(Handler::new());
        let ep = Endpoint::bind(presets::N0).await.map_err(|e| P2PError::during("Could not connect to hardware", P2PError::BindError(e.to_string())))?;
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();

        let conn = tokio::time::timeout(
            std::time::Duration::from_secs(5), 
            ep.connect(id, ALPN)
        )
            .await
            .map_err(|_e|
                P2PError::during("Error connecting to server", P2PError::Timeout))?
            .map_err(|e| 
                P2PError::during("Error creating connection with Iroh relays. If this persists, ensure there is no middleman in your TLS connections or use a VPN.", P2PError::CreateConnectionError(e.to_string()))
            )?;


        let (send, recv) = conn.open_bi().await.map_err(|e| P2PError::during("Error creating connection with server", P2PError::CreateConnectionError(e.to_string())))?;

        let this = Self {
            handler: handler,
            router: router,
            send: Mutex::new(Some(send)),
            recv: Mutex::new(Some(recv)),
        };

        this.send(ALPN).await?;

        Ok(this)
    }

    pub async fn send(&self, message: &[u8]) -> Result<(), P2PError> {
        let mut send_lock = self.send.lock().await;
        let send = send_lock.as_mut().ok_or(P2PError::during("Connection lost", P2PError::ConnectionNotFound))?;

        let len: [u8; 4] = (message.len() as u32).to_be_bytes();

        send.write_all(&len)   .await.map_err(|_| P2PError::during("Connection lost", P2PError::ConnectionNotFound))?;
        send.write_all(message).await.map_err(|_| P2PError::during("Connection lost", P2PError::ConnectionNotFound))?;
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<u8>, P2PError> {
        let mut recv_lock = self.recv.lock().await;
        let recv = recv_lock.as_mut().ok_or(P2PError::during("Connection lost", P2PError::ConnectionNotFound))?;

        let mut byte = [0u8; 1];
        let mut length: u32 = 0;
        
        // get length of message (first 4 bytes)
        for _ in 0..4 {
            match recv.read(&mut byte).await.map_err(|_| P2PError::during("Connection lost", P2PError::ConnectionNotFound))? {
                None => return Ok(vec![]),
                Some(_) => {
                    // println!("Read byte {:?}", byte);
                    length <<= 8;
                    length |= byte[0] as u32;
                }
            }
        }

        let mut bytes: Vec<u8> = vec![];
        bytes.resize(length as usize, 0);

        let _ = recv.read_exact(&mut bytes[..]).await;

        Ok(bytes)
    }

    pub async fn close(&mut self) {
        let _ = self.router.shutdown().await;
    }
}