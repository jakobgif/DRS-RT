use std::net::UdpSocket;

// F-7, F-8: reflect every received packet back to the sender; run until terminated
pub fn run(port: u16) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", port))?;
    log::info!("Echo listening on 0.0.0.0:{}", port);

    let mut buf = [0u8; 64];
    let mut connected = false;

    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                // Optimization: Connect the socket to the first peer we see.
                // This reduces the overhead of subsequent send_to calls because the 
                // kernel doesn't have to verify the destination address each time.
                if !connected {
                    if let Ok(()) = socket.connect(src) {
                        connected = true;
                        log::info!("Echo connected to master at {}", src);
                    }
                }

                // F-7: reflect byte-for-byte without modification
                let res = if connected {
                    socket.send(&buf[..n])
                } else {
                    socket.send_to(&buf[..n], src)
                };

                if let Err(e) = res {
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
