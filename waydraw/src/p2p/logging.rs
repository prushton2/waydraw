use std::sync::{Arc, Mutex, OnceLock};
use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter, Registry};

#[derive(Clone, Default)]
pub struct StringWriter(pub Arc<Mutex<String>>);

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

pub struct Logging {
    filter: reload::Handle<EnvFilter, Registry>,
    pub logs: StringWriter,
}

static LOGGING: OnceLock<Logging> = OnceLock::new();

pub fn logging() -> &'static Logging {
    LOGGING.get_or_init(|| {
        let logs = StringWriter::default();
        let (filter, handle) = reload::Layer::new(EnvFilter::new("off"));
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(logs.clone()).with_ansi(false))
            .try_init();
        Logging { filter: handle, logs }
    })
}

impl Logging {
    pub fn enable(&self) {
        self.logs.0.lock().unwrap().clear(); // drop old logs from the last run
        let _ = self.filter.modify(|f| *f = EnvFilter::new("iroh=warn"));
    }

    pub fn disable(&self) {
        let _ = self.filter.modify(|f| *f = EnvFilter::new("off"));
    }
}
