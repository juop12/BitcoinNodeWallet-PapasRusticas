use std::net::{SocketAddr, ToSocketAddrs};

const DNS_PORT: u16 = 53; //DNS PORT

/// Struct that represents the bitcoin node
struct Node {
    version: i32,
}

impl Node {
    pub fn new() -> Node {
        Node { version: 70015 }
    }
    /// Receives a dns address as a String and returns a Vector that contains all the addresses
    /// returned by the dns. If an error occured (for example, the dns address is not valid), it
    /// returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT, which is set to 53 by default
    fn peer_discovery(&self, dns: String) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();
        match (dns, DNS_PORT).to_socket_addrs() {
            Ok(address_iter) => {
                for address in address_iter {
                    socket_address_vector.push(address);
                }
            }
            Err(_) => {}
        }
        socket_address_vector
    }
}
/*
 fn connect_user_to_server(dns: String) {
     let addrs_iter = (dns, DNS_PORT).to_socket_addrs().unwrap();

     TcpStream::connect(&addrs[..])
}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_peer_discovery_fails_when_receiving_invalid_dns() {
        let node = Node::new();
        let address_vector = node.peer_discovery(String::from("does_not_exist"));
        assert!(address_vector.is_empty());
    }

    #[test]
    fn test_2_peer_discovery_returns_ip_vector_when_receiving_valid_dns() {
        let node = Node::new();
        let address_vector =
            node.peer_discovery(String::from("testnet-seed.bitcoin.jonasschnelli.ch"));
        assert!(!address_vector.is_empty());
    }
}
