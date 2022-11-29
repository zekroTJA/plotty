use crate::conf;
use minecraft_client_rs::{Client, Message};
use std::error;

type Error = Box<dyn error::Error>;

pub struct Rcon {
    cfg: conf::Rcon,
}

impl Rcon {
    pub fn new(cfg: &conf::Rcon) -> Result<Self, Error> {
        Ok(Self { cfg: cfg.clone() })
    }

    pub fn get_conn(&self) -> Result<Conn, Error> {
        let mut client = Client::new(self.cfg.address.to_owned())?;
        client.authenticate(self.cfg.password.to_owned())?;
        Ok(Conn(client))
    }
}

pub struct Conn(Client);

impl Drop for Conn {
    fn drop(&mut self) {
        log::debug!("Connection closed");
        if let Err(err) = self.0.close() {
            log::error!("Failed closing connection: {err}");
        }
    }
}

impl Conn {
    pub fn cmd(&mut self, cmd: &str) -> Result<Message, Error> {
        self.0.send_command(cmd.to_owned())
    }
}
