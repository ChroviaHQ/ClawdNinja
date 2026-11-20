use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use clawdninja_core::proxy_url;

use crate::error::CommandError;

const DEFAULT_PORT: NonZeroU16 = NonZeroU16::new(1080).unwrap();
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

pub struct Egress {
    path: PathBuf,
    port: Mutex<NonZeroU16>,
}

impl Egress {
    pub fn load(path: PathBuf) -> Self {
        let port = fs::read_to_string(&path)
            .ok()
            .and_then(|text| text.trim().parse().ok())
            .unwrap_or(DEFAULT_PORT);
        Self { path, port: Mutex::new(port) }
    }

    pub fn port(&self) -> NonZeroU16 {
        *self.port.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn connect(&self, port: NonZeroU16) -> Result<String, CommandError> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port.get()));
        TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).map_err(|error| {
            CommandError::new("egress_unreachable", format!("{address}: {error}"))
        })?;
        self.remember(port);
        Ok(proxy_url(port))
    }

    fn remember(&self, port: NonZeroU16) {
        let mut saved = self.port.lock().unwrap_or_else(PoisonError::into_inner);
        if *saved == port {
            return;
        }
        if let Err(error) = persist(&self.path, port) {
            tracing::warn!(%error, "cannot save egress port");
        }
        *saved = port;
    }
}

fn persist(path: &Path, port: NonZeroU16) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, port.to_string())
}
