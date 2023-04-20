use chrono::{DateTime, Utc};
use rand::Rng;

const NODE_NETWORK: u64 = 0x01;
const LOCAL_HOST: [u8; 16] = 127.0.0.1;
const LOCAL_PORT: u16 = 1001;

/// Contains all necessary fields, for sending a version message needed for doing a handshake among nodes
struct VersionMessage {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv_services: u64,
    receiver_address: [u8; 16],
    receiver_port: u16,
    addr_sender_services: u64,
    sender_address: [u8; 16],
    sender_port: u16,
    nonce: u64,
    user_agent_length: u8,
    //user_agent: String,
    start_height: i32,
    relay: bool,
}

impl VersionMessage {
    /// Constructor for the struct VersionMessage, receives a version and a reciever address (which
    /// includes both the ip and port) and returns an instance of a VersionMessage with all its 
    /// necesary attributes initialized, the optional ones are left in blank
    pub fn new(version: i32, receiver_address :SocketAddr) -> VersionMessage {
        VersionMessage {
            version,
            services: NODE_NETWORK,
            timestamp: Utc::now().timestamp(),
            addr_recv_services: 0, //Como no sabemos que servicios admite el nodo asumimos que no admite ningun servicio
            receiver_address.ip().to_bytes(),
            receiver_port: receiver_address.port().to_bytes(),
            addr_sender_services: NODE_NETWORK,
            sender_address: LOCAL_HOST,
            sender_port: LOCAL_PORT,
            nonce: Rng::gen(),
            user_agent_length: 0,
            //user_agent,   no ponemos el user agent,porque entendemos que nadie nos conoce, a nadie le va a interesar saber en que version esta papas rusticas 0.0.1
            start_height: 0,
            relay: true,
        }
    }
}