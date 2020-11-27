use std::time::Duration;
use std::net::IpAddr;

struct Process<'a> {
    coordinator: &'a Self,
    peers: Vec<PeerInfo>,
    id: u8,
    address: IpAddr,
    response_timeout: Duration,
    hearbeat_timeout: Duration,
}

struct PeerInfo{
    id: u8,
    address: String
}

impl<'a> Process<'a> {
    pub fn new(peer_addr_strs: Vec<String>) -> Self {unimplemented!()}

    /// start will run the process forever
    pub fn start(self) {unimplemented!()}
}
