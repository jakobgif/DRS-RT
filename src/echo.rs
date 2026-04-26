use std::net::UdpSocket;

// F-7, F-8: reflect every received packet back to the sender; run until terminated
pub fn run(port: u16) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", port))?;
    log::info!("Echo listening on 0.0.0.0:{}", port);

    let mut buf = [0u8; 64];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                // F-7: reflect byte-for-byte without modification
                if let Err(e) = socket.send_to(&buf[..n], src) {
                    log::warn!("Echo send error to {}: {}", src, e);
                }
            }
            Err(e) => {
                // NF-6: log and continue
                log::error!("Echo recv error: {}", e);
            }
        }
    }
}
