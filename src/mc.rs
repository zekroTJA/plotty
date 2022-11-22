use crate::conf;
use minecraft_client_rs::{Client, Message};
use std::error;

type Error = Box<dyn error::Error>;

pub struct Rcon {
    client: Client,
}

impl Rcon {
    pub fn new(cfg: &conf::Rcon) -> Result<Self, Error> {
        let mut client = Client::new(cfg.address.to_owned())?;
        client.authenticate(cfg.password.to_owned())?;

        Ok(Self { client })
    }

    pub fn cmd(&mut self, cmd: &str) -> Result<Message, Error> {
        self.client.send_command(cmd.to_owned())
    }
}
