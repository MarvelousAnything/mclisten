use color_eyre::eyre::{eyre, Result};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use mc_varint::VarIntRead;

static PACKET_DATA: &[u8] = include_bytes!("data/packets/0758_1.18.2.csv");

lazy_static! {
    static ref PACKET_MAP: HashMap<(String, String, u8), PacketData> = {
        let mut map = HashMap::new();
        let mut rdr = csv::Reader::from_reader(PACKET_DATA);
        let mut ident = 0;
        let mut last_section = (String::new(), String::new());
        for result in rdr.deserialize() {
            let packet: PacketData = result.expect("Failed to deserialize packet");
            let section = (packet.protocol_mode.to_string(), packet.packet_direction.to_string());
            if section != last_section {
                ident = 0;
            }
            last_section = section;
            let key = (packet.protocol_mode.to_string(), packet.packet_direction.to_string(), ident);
            map.insert(
                key,
                packet,
            );
            ident += 1;
        }
        map
    };
}

#[derive(Debug, Deserialize, Clone)]
struct PacketData {
    protocol_mode: String,
    packet_direction: String,
    packet_name: String,
}

#[derive(Debug)]
pub struct Packet {
    pub id: u8,
    pub size: u8,
    pub version: u8,
    data: &'static PacketData,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn from_buffer(buffer: &[u8]) -> Result<()> {
        let cursor = &mut io::Cursor::new(buffer);
        debug!("Parsing packet from buffer: [{}, {}, ...]", buffer[0], buffer[1]);
        let size = cursor.read_var_u32()? as u8;
        // debug!("Read packet with size: {:?}", size);
        let id = cursor.read_var_u32()? as u8;
        // debug!("Read packet with id: {:x?}", id);
        let payload = cursor.remaining_slice();
        // debug!("Read packet with payload: {:x?}", payload);
        // let data = PACKET_MAP
        //     .get(&id)
        //     .ok_or_else(|| eyre!("Packet with id {:?} not found", id))?;
        // Ok(Self {
        //     id,
        //     size,
        //     data,
        //     payload,
        // })
        Ok(())
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {:x?}",
            self.id,
            self.size,
            self.data.packet_name,
            self.data.protocol_mode,
            self.data.packet_direction,
            self.payload
        )
    }
}

#[derive(Debug)]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
}

#[derive(Debug, Clone)]
pub enum ProtocolMode {
    Handshake = 0,
    Status,
    Login,
    Play,
}

pub struct PacketParser {
    pub protocol_mode: ProtocolMode,
    pub packet_direction: PacketDirection,
}

impl Default for PacketParser {
    fn default() -> Self {
        Self {
            protocol_mode: ProtocolMode::Handshake,
            packet_direction: PacketDirection::Serverbound,
        }
    }
}

impl PacketParser {
    pub fn check_protocol_mode_switch(&self, mode: &ProtocolMode) -> Result<()> {
        match (&self.protocol_mode, mode) {
            (ProtocolMode::Handshake, ProtocolMode::Status) => Ok(()),
            (ProtocolMode::Handshake, ProtocolMode::Login) => Ok(()),
            (ProtocolMode::Login, ProtocolMode::Login) => Ok(()),
            _ => Err(eyre!("Invalid protocol mode switch {:?} -> {:?}", self.protocol_mode, mode)),
        }
    }

    pub fn switch_protocol_mode(&mut self, mode: &ProtocolMode) -> Result<()> {
        self.check_protocol_mode_switch(mode)?;
        self.protocol_mode = mode.to_owned();
        Ok(())
    }
}