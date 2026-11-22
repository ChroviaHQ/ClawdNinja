use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::num::NonZeroU16;
use std::path::PathBuf;
use std::time::Duration;

use clawdninja_core::proxy_url;

use crate::error::CommandError;
use crate::preference::Preference;

const DEFAULT_PORT: NonZeroU16 = NonZeroU16::new(1080).unwrap();
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

pub struct Egress(Preference<NonZeroU16>);

impl Egress {
    pub fn load(path: PathBuf) -> Self {
        Self(Preference::load(path, DEFAULT_PORT))
    }

    pub fn port(&self) -> NonZeroU16 {
        self.0.get()
    }

    pub fn connect(&self, port: NonZeroU16) -> Result<String, CommandError> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port.get()));
        TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).map_err(|error| {
            CommandError::new("egress_unreachable", format!("{address}: {error}"))
        })?;
        self.0.set(port);
        Ok(proxy_url(port))
    }
}
