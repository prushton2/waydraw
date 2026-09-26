use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use anyhow::Result;
use iroh::{Endpoint, PublicKey, SecretKey, endpoint::{Connection, RecvStream, SendStream, presets}, protocol::{AcceptError, ProtocolHandler, Router}};
use tracing_subscriber::EnvFilter;

const HELLO: &[u8] = b"hello";

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum P2PError {
    ErrorDuring(String, Box<P2PError>),
    BindError(String),
    AcceptConnectionError(String),
    CreateConnectionError(String),
    ReadError(String),
    InputError(String),
    ConnectionNotFound,
    PoisonedMutex,
    Timeout,
}

impl std::fmt::Display for P2PError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
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
            P2PError::ReadError(r) =>                     format!("{} (ReadError)", r),
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
    pub async fn init(secret_key: SecretKey) -> Result<(Self, PublicKey), P2PError> {
        let handler = Box::new(Handler::new());
        let logs = StringWriter::default();

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("iroh=warn"))
            .with_writer(logs.clone())
            .with_ansi(false) // filter out color codes
            .init();

        let ep = Endpoint::builder(presets::N0)
            .secret_key(secret_key)
            .bind()
            .await
            .map_err(|e| P2PError::during("Could not connect to hardware", P2PError::BindError(e.to_string())))?;
        
        let router = Router::builder(ep.clone()).accept(HELLO, handler.clone()).spawn();

        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            ep.online()
        ).await
            .map_err(|_| {
                let lock = logs.0.lock().unwrap();
                let mut split = lock.split("\n");
                P2PError::during(format!("Error connecting to Iroh: {}", split.nth(0).unwrap()).as_str(), P2PError::Timeout)
            }
        )?;

        // let _ = filter_handle.modify(|f| *f = EnvFilter::new("off"));

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
        let router = Router::builder(ep.clone()).accept(HELLO, handler.clone()).spawn();

        let conn = tokio::time::timeout(
            std::time::Duration::from_secs(5), 
            ep.connect(id, HELLO)
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

        this.send(HELLO).await?;

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

        let mut bytes = [0u8; 4];
        
        recv.read_exact(&mut bytes)
            .await
            .map_err(|e| P2PError::during("Reading message length", P2PError::ReadError(e.to_string())))?;

        let length = u32::from_be_bytes(bytes);

        let mut bytes: Vec<u8> = vec![];
        bytes.resize(length as usize, 0);

        tokio::time::timeout(std::time::Duration::from_secs(5), recv.read_exact(&mut bytes[..]))
            .await
            .map_err(|_| P2PError::ErrorDuring(format!("Reading exactly {} bytes", length), Box::new(P2PError::Timeout)))?
            .map_err(|e| P2PError::during(format!("Reading exactly {} bytes", length).as_str(), P2PError::ReadError(e.to_string())))?;

        Ok(bytes)
    }

    pub async fn close(&mut self) {
        let _ = self.router.shutdown().await;
    }
}

#[derive(Clone, Default)]
struct StringWriter(Arc<std::sync::Mutex<String>>);

impl std::io::Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().push_str(&String::from_utf8_lossy(buf));
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for StringWriter {
    type Writer = Self;
    fn make_writer(&'a self) -> Self::Writer { self.clone() }
}

fn key_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config"));
    base.join("waydraw").join("secret_key")
}

pub fn load_or_create_secret_key() -> Result<SecretKey, P2PError> {
    let path = key_path();

    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return Ok(SecretKey::from_bytes(&bytes));
        }
    }

    let key = SecretKey::generate();
    let err = |e: std::io::Error| P2PError::during("Saving secret key", P2PError::InputError(e.to_string()));
    std::fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
    std::fs::write(&path, key.to_bytes()).map_err(err)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(err)?;
    }
    Ok(key)
}