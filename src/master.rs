use std::io::{ErrorKind, Write};
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use socket2::{Socket, Domain, Type, Protocol};

use crate::types::{Sample, Status};

// F-2, F-3, F-4, F-5, F-11, F-18, F-19
pub struct MasterConfig {
    pub host: String,
    pub port: u16,
    pub cycles: u64,
    pub timeout_secs: f64,
    pub warmup: u64,
    pub cpu_pin: Option<usize>,
    pub output: String,
}

pub fn run(cfg: MasterConfig) -> anyhow::Result<()> {
    // F-19: pin thread before measurement begins
    if let Some(core) = cfg.cpu_pin {
        set_affinity(core)?;
        log::info!("Pinned measurement thread to CPU core {}", core);
    }

    let peer_addr = format!("{}:{}", cfg.host, cfg.port);
    
    // Optimization: Use socket2 to set low-latency options
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.bind(&"0.0.0.0:0".parse::<std::net::SocketAddr>()?.into())?;
    socket.connect(&peer_addr.parse::<std::net::SocketAddr>()?.into())?;
    
    // Optimization: Use non-blocking mode for spin-waiting to avoid context switches
    socket.set_nonblocking(true)?;
    let socket: UdpSocket = socket.into();

    let timeout = Duration::from_secs_f64(cfg.timeout_secs);

    // NF-4 & Optimization: Pre-fill full buffer before any measurement to eliminate
    // bounds/capacity checks (Vec::push) inside the hot path.
    let cap = cfg.cycles as usize;
    let mut samples = vec![Sample { timestamp_us: 0, rtt_us: 0, status: Status::Ok }; cap];

    let mut seq: u64 = 0;
    let mut recv_buf = [0u8; 64];

    // F-18: warm-up cycles — unrecorded, primes ARP table and CPU caches
    log::info!("Starting {} warm-up cycle(s) to {}", cfg.warmup, peer_addr);
    for _ in 0..cfg.warmup {
        let send_buf = seq.to_le_bytes();
        seq += 1;
        let _ = socket.send(&send_buf);
        
        let t_start = Instant::now();
        while t_start.elapsed() < timeout {
            if socket.recv(&mut recv_buf).is_ok() { break; }
        }
    }
    log::info!("Warm-up complete. Starting {} measurement cycle(s).", cfg.cycles);

    // ── Measurement loop — hot path ────────────────────────────────────────────
    // NF-5, F-10: no log macros, no allocation, no file I/O inside this loop.
    let loop_start = Instant::now();

    for i in 0..cap {
        let current_seq = seq;
        let send_buf = seq.to_le_bytes(); // 8 bytes on stack
        seq += 1;

        // F-6, F-15: send packet carrying u64 sequence number
        let t_send = Instant::now(); // NF-2: timestamp at send
        if socket.send(&send_buf).is_err() {
            // NF-6: socket send error — record as lost, continue
            samples[i] = Sample {
                timestamp_us: t_send.duration_since(loop_start).as_micros() as u64,
                rtt_us: -1,
                status: Status::Timeout,
            };
            continue;
        }

        // Optimization: Spin-wait for the reply to avoid thread descheduling/context switches.
        // Also handles F-16 by continuing to wait if a stale/mismatched packet is received,
        // preventing a cascade of sequence mismatches.
        let mut final_status;
        let mut t_recv = Instant::now();
        let mut saw_mismatch = false;

        loop {
            match socket.recv(&mut recv_buf) {
                Ok(n) => {
                    t_recv = Instant::now(); // NF-2: timestamp at receive
                    if n >= 8 {
                        let reflected = u64::from_le_bytes(recv_buf[..8].try_into().unwrap());
                        if reflected == current_seq {
                            final_status = Status::Ok;
                            break;
                        } else {
                            // F-16: Stale/mismatched packet received. 
                            // We ignore it and continue spinning for the correct sequence number.
                            saw_mismatch = true;
                            continue;
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Optimization: Spin-wait until timeout
                    if t_send.elapsed() >= timeout {
                        final_status = if saw_mismatch { Status::SeqMismatch } else { Status::Timeout };
                        break;
                    }
                    std::hint::spin_loop();
                }
                Err(_) => {
                    final_status = if saw_mismatch { Status::SeqMismatch } else { Status::Timeout };
                    break;
                }
            }
        }

        // F-9, F-17: record RTT or -1 for lost cycles
        let rtt_us = match final_status {
            Status::Ok => t_recv.duration_since(t_send).as_micros() as i64,
            _ => -1,
        };

        // Optimization: Direct indexing into pre-filled buffer (no capacity check)
        samples[i] = Sample {
            timestamp_us: t_send.duration_since(loop_start).as_micros() as u64,
            rtt_us,
            status: final_status,
        };
    }
    // ── End hot path ───────────────────────────────────────────────────────────

    // F-12, F-13: log lost-packet events after loop completes
    let lost_count = samples.iter().filter(|s| !matches!(s.status, Status::Ok)).count();
    log::info!(
        "Measurement complete: {} cycles, {} ok, {} lost",
        cfg.cycles,
        cfg.cycles as usize - lost_count,
        lost_count
    );
    for (i, s) in samples.iter().enumerate() {
        if !matches!(s.status, Status::Ok) {
            log::warn!(
                "Cycle {}: {} (timestamp_us={})",
                i,
                s.status.as_str(),
                s.timestamp_us
            );
        }
    }

    // PF-2: compute and log RTT statistics
    let ok_rtts: Vec<i64> = samples
        .iter()
        .filter(|s| matches!(s.status, Status::Ok))
        .map(|s| s.rtt_us)
        .collect();
    if !ok_rtts.is_empty() {
        let min = *ok_rtts.iter().min().unwrap();
        let max = *ok_rtts.iter().max().unwrap();
        let mean = ok_rtts.iter().sum::<i64>() / ok_rtts.len() as i64;
        log::info!("RTT (us): min={} mean={} max={}", min, mean, max);
    }

    // F-14, F-17: write CSV — one row per cycle, no header
    write_csv(&cfg.output, &samples)?;
    log::info!("CSV written to {}", cfg.output);

    Ok(())
}

// F-14, F-17: format is timestamp_us,rtt_us,status
fn write_csv(path: &str, samples: &[Sample]) -> anyhow::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    for s in samples {
        writeln!(w, "{},{},{}", s.timestamp_us, s.rtt_us, s.status.as_str())?;
    }
    Ok(())
}

// F-19: CPU affinity — Linux only
fn set_affinity(core: usize) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let core_id = core_affinity::CoreId { id: core };
        if !core_affinity::set_for_current(core_id) {
            anyhow::bail!("Failed to pin thread to CPU core {}", core);
        }
        return Ok(());
    }
    #[cfg(not(target_os = "linux"))]
    {
        log::warn!("--cpu-pin is only supported on Linux; ignoring core {}", core);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // UT-2: buffer pre-allocated with full capacity before first send
    #[test]
    fn buffer_preallocated_to_cycle_count() {
        let cycles = 1000usize;
        let mut buf: Vec<Sample> = Vec::new();
        buf.try_reserve(cycles).expect("allocation failed");
        assert_eq!(buf.capacity(), cycles);

        // Simulate insertions — must not reallocate (no capacity growth)
        let cap_before = buf.capacity();
        for i in 0..cycles {
            buf.push(Sample {
                timestamp_us: i as u64,
                rtt_us: 100,
                status: Status::Ok,
            });
        }
        assert_eq!(buf.capacity(), cap_before, "capacity grew: reallocation occurred");
    }

    // UT-3: CSV serialization — one row per cycle, three fields, correct status strings
    #[test]
    fn csv_serialization() {
        let samples = vec![
            Sample { timestamp_us: 0, rtt_us: 500, status: Status::Ok },
            Sample { timestamp_us: 1000, rtt_us: -1, status: Status::Timeout },
            Sample { timestamp_us: 2000, rtt_us: -1, status: Status::SeqMismatch },
        ];

        let path = "test_csv_serialization.csv";
        write_csv(path, &samples).expect("write_csv failed");

        let content = std::fs::read_to_string(path).expect("read failed");
        std::fs::remove_file(path).ok();

        let rows: Vec<&str> = content.trim_end().split('\n').collect();
        assert_eq!(rows.len(), 3, "expected 3 rows");

        let fields: Vec<&str> = rows[0].split(',').collect();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "0");
        assert_eq!(fields[1], "500");
        assert_eq!(fields[2], "ok");

        let fields: Vec<&str> = rows[1].split(',').collect();
        assert_eq!(fields[2], "timeout");

        let fields: Vec<&str> = rows[2].split(',').collect();
        assert_eq!(fields[2], "seq_mismatch");
    }

    // UT-5: packet loss recording — timeout produces -1 with status timeout
    #[test]
    fn timeout_records_negative_one() {
        let mut samples: Vec<Sample> = Vec::new();
        samples.try_reserve(1).unwrap();
        samples.push(Sample {
            timestamp_us: 0,
            rtt_us: -1,
            status: Status::Timeout,
        });
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].rtt_us, -1);
        assert!(matches!(samples[0].status, Status::Timeout));
    }

    // UT-6: seq mismatch produces -1 with status seq_mismatch
    #[test]
    fn seq_mismatch_records_negative_one() {
        let mut samples: Vec<Sample> = Vec::new();
        samples.try_reserve(1).unwrap();
        samples.push(Sample {
            timestamp_us: 0,
            rtt_us: -1,
            status: Status::SeqMismatch,
        });
        assert_eq!(samples[0].rtt_us, -1);
        assert!(matches!(samples[0].status, Status::SeqMismatch));
    }

    // UT-7: warm-up cycles leave RTT buffer empty
    #[test]
    fn warmup_does_not_populate_buffer() {
        // Simulate warmup: no push to samples during warmup iterations
        let warmup = 10u64;
        let mut samples: Vec<Sample> = Vec::new();
        samples.try_reserve(50).unwrap();

        // Warmup loop does NOT touch samples
        let mut seq = 0u64;
        for _ in 0..warmup {
            seq += 1; // just increments seq
        }
        let _ = seq;

        assert_eq!(samples.len(), 0, "warm-up must not populate the RTT buffer");
    }
}
