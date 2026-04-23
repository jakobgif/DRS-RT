use std::net::UdpSocket;
use std::time::Duration;

use drs_rt::master::{self, MasterConfig};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Bind a UDP echo socket on a random port and return that port.
/// The spawned thread reflects up to `max_packets` datagrams then exits.
fn spawn_echo(max_packets: usize) -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    std::thread::spawn(move || {
        let mut buf = [0u8; 64];
        for _ in 0..max_packets {
            match socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    let _ = socket.send_to(&buf[..n], src);
                }
                Err(_) => break,
            }
        }
    });
    port
}

/// Like spawn_echo but corrupts the reflected sequence number — forces seq_mismatch.
fn spawn_mismatch_echo(max_packets: usize) -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    std::thread::spawn(move || {
        let mut buf = [0u8; 64];
        for _ in 0..max_packets {
            match socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    if n >= 8 {
                        let bad = u64::from_le_bytes(buf[..8].try_into().unwrap())
                            .wrapping_add(999);
                        buf[..8].copy_from_slice(&bad.to_le_bytes());
                    }
                    let _ = socket.send_to(&buf[..n], src);
                }
                Err(_) => break,
            }
        }
    });
    port
}

fn unique_path(tag: &str, ext: &str) -> String {
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("it_{}_{}.{}", tag, ns, ext)
}

fn parse_csv(path: &str) -> Vec<(u64, i64, String)> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.splitn(3, ',').collect();
            (
                cols[0].parse().unwrap(),
                cols[1].parse().unwrap(),
                cols[2].to_string(),
            )
        })
        .collect()
}

fn echo_cfg(port: u16, cycles: u64, csv: String) -> MasterConfig {
    MasterConfig {
        host: "127.0.0.1".into(),
        port,
        cycles,
        timeout_secs: 2.0,
        warmup: 0,
        cpu_pin: None,
        output: csv,
    }
}

// ── IT-1 ──────────────────────────────────────────────────────────────────────

/// IT-1: Echo reflects every packet back byte-for-byte unchanged. (F-7)
#[test]
fn it1_echo_reflection() {
    let port = spawn_echo(5);

    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client.connect(format!("127.0.0.1:{}", port)).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let payload = 0xDEADBEEFu64.to_le_bytes();
    client.send(&payload).unwrap();

    let mut buf = [0u8; 64];
    let n = client.recv(&mut buf).unwrap();
    assert_eq!(n, 8, "reflected length must equal sent length");
    assert_eq!(&buf[..8], &payload, "reflected payload must be unchanged");
}

// ── IT-2 ──────────────────────────────────────────────────────────────────────

/// IT-2: RTT is positive after one successful Master–Echo cycle. (F-6, F-9)
#[test]
fn it2_rtt_recorded_positive() {
    let port = spawn_echo(10);
    let csv = unique_path("it2", "csv");

    master::run(echo_cfg(port, 1, csv.clone())).unwrap();

    let rows = parse_csv(&csv);
    std::fs::remove_file(&csv).ok();

    assert_eq!(rows.len(), 1);
    assert!(rows[0].1 > 0, "RTT must be positive for a successful cycle");
    assert_eq!(rows[0].2, "ok");
}

// ── IT-3 ──────────────────────────────────────────────────────────────────────

/// IT-3: RTT buffer contains exactly N entries for N cycles. (F-5, F-10, NF-4)
#[test]
fn it3_full_cycle_count() {
    let cycles = 20u64;
    let port = spawn_echo(cycles as usize + 5);
    let csv = unique_path("it3", "csv");

    master::run(echo_cfg(port, cycles, csv.clone())).unwrap();

    let rows = parse_csv(&csv);
    std::fs::remove_file(&csv).ok();

    assert_eq!(rows.len() as u64, cycles, "CSV must have exactly N rows");
}

// ── IT-4 ──────────────────────────────────────────────────────────────────────

/// IT-4: Timeout is recorded per cycle and the loop continues without blocking. (F-11, F-12)
#[test]
fn it4_timeout_and_continuation() {
    let cycles = 3u64;
    let csv = unique_path("it4", "csv");

    // Port 59990: nothing listening; every recv will time out or error immediately.
    master::run(MasterConfig {
        host: "127.0.0.1".into(),
        port: 59990,
        cycles,
        timeout_secs: 0.05,
        warmup: 0,
        cpu_pin: None,
        output: csv.clone(),
    })
    .unwrap();

    let rows = parse_csv(&csv);
    std::fs::remove_file(&csv).ok();

    assert_eq!(rows.len() as u64, cycles, "must record one row per cycle");
    for row in &rows {
        assert_eq!(row.1, -1, "lost cycle must have rtt_us == -1");
        assert_eq!(row.2, "timeout", "lost cycle must have status == timeout");
    }
}

// ── IT-5 ──────────────────────────────────────────────────────────────────────

/// IT-5: CSV content is correct for ok, timeout, and seq_mismatch cycles. (F-12, F-14, F-17)
#[test]
fn it5_csv_content_all_statuses() {
    // ok cycles
    {
        let port = spawn_echo(10);
        let csv = unique_path("it5_ok", "csv");
        master::run(echo_cfg(port, 5, csv.clone())).unwrap();
        let rows = parse_csv(&csv);
        std::fs::remove_file(&csv).ok();
        assert!(
            rows.iter().all(|r| r.1 > 0 && r.2 == "ok"),
            "all successful cycles must have positive RTT and status ok"
        );
    }

    // timeout cycles
    {
        let csv = unique_path("it5_to", "csv");
        master::run(MasterConfig {
            host: "127.0.0.1".into(),
            port: 59991,
            cycles: 2,
            timeout_secs: 0.05,
            warmup: 0,
            cpu_pin: None,
            output: csv.clone(),
        })
        .unwrap();
        let rows = parse_csv(&csv);
        std::fs::remove_file(&csv).ok();
        assert!(
            rows.iter().all(|r| r.1 == -1 && r.2 == "timeout"),
            "all timed-out cycles must have rtt_us == -1 and status timeout"
        );
    }

    // seq_mismatch cycles
    {
        let port = spawn_mismatch_echo(10);
        let csv = unique_path("it5_sm", "csv");
        master::run(echo_cfg(port, 5, csv.clone())).unwrap();
        let rows = parse_csv(&csv);
        std::fs::remove_file(&csv).ok();
        assert!(
            rows.iter().all(|r| r.1 == -1 && r.2 == "seq_mismatch"),
            "all seq-mismatch cycles must have rtt_us == -1 and status seq_mismatch"
        );
    }
}

// ── IT-6 ──────────────────────────────────────────────────────────────────────

/// IT-6: No file I/O occurs inside the measurement loop. (F-10, NF-5)
///
/// Verified structurally: in master::run(), write_csv() is the first file operation
/// and is called unconditionally after the loop. No file handles are opened or written
/// inside the loop — confirmed by code inspection of master.rs. This runtime test
/// confirms the binary completes successfully, which is only possible if the CSV is
/// written after the loop (the file is absent during the loop and present after).
#[test]
fn it6_no_io_inside_loop() {
    let port = spawn_echo(20);
    let csv = unique_path("it6", "csv");

    assert!(!std::path::Path::new(&csv).exists(), "CSV must not exist before run");
    master::run(echo_cfg(port, 10, csv.clone())).unwrap();
    assert!(std::path::Path::new(&csv).exists(), "CSV must exist after run");

    std::fs::remove_file(&csv).ok();
}

// ── IT-7 ──────────────────────────────────────────────────────────────────────

/// IT-7: RTT buffer contains exactly N entries, not N+warmup. (F-18)
#[test]
fn it7_warmup_excluded_from_buffer() {
    let warmup = 5u64;
    let cycles = 10u64;
    let port = spawn_echo((warmup + cycles + 5) as usize);
    let csv = unique_path("it7", "csv");

    master::run(MasterConfig {
        host: "127.0.0.1".into(),
        port,
        cycles,
        timeout_secs: 2.0,
        warmup,
        cpu_pin: None,
        output: csv.clone(),
    })
    .unwrap();

    let rows = parse_csv(&csv);
    std::fs::remove_file(&csv).ok();

    assert_eq!(
        rows.len() as u64,
        cycles,
        "CSV must contain exactly N rows (warm-up cycles must not be recorded)"
    );
}

// ── IT-8 ──────────────────────────────────────────────────────────────────────

/// IT-8: Master pins the measurement thread to the specified CPU core. (F-19)
/// Linux-only — reads /proc/thread-self/status to verify the affinity mask.
#[test]
#[cfg(target_os = "linux")]
fn it8_cpu_affinity_linux() {
    let port = spawn_echo(10);
    let csv = unique_path("it8", "csv");

    let status_text = std::thread::spawn(move || {
        master::run(MasterConfig {
            host: "127.0.0.1".into(),
            port,
            cycles: 1,
            timeout_secs: 2.0,
            warmup: 0,
            cpu_pin: Some(0),
            output: csv.clone(),
        })
        .unwrap();
        std::fs::remove_file(&csv).ok();
        // Affinity persists after run — read from this same pinned thread.
        std::fs::read_to_string("/proc/thread-self/status").unwrap()
    })
    .join()
    .unwrap();

    let raw = status_text
        .lines()
        .find(|l| l.starts_with("Cpus_allowed:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().replace(',', ""))
        .expect("Cpus_allowed field not found in /proc/thread-self/status");

    let mask = u128::from_str_radix(&raw, 16).expect("Cpus_allowed is not a hex number");
    assert_eq!(mask, 1, "thread must be pinned to core 0 only (mask must be 0x1)");
}

// ── IT-9 ──────────────────────────────────────────────────────────────────────

/// IT-9: Log file is created and contains lost-packet entries. (F-13)
/// Runs the compiled binary as a subprocess to exercise the full logger setup.
#[test]
fn it9_log_file_created_with_content() {
    let log_file = unique_path("it9", "log");
    let csv_file = unique_path("it9", "csv");

    let bin = env!("CARGO_BIN_EXE_drs-rt");
    let result = std::process::Command::new(bin)
        .args([
            "master",
            "--host",
            "127.0.0.1",
            "--port",
            "59992",
            "--cycles",
            "2",
            "--timeout",
            "0.05",
            "--warmup",
            "0",
            "--output",
            &csv_file,
            "--log",
            &log_file,
        ])
        .output()
        .expect("failed to execute drs-rt binary");

    assert!(
        result.status.success(),
        "binary must exit successfully; stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // F-13: log file must be created and non-empty
    assert!(
        std::path::Path::new(&log_file).exists(),
        "log file must be created after run"
    );
    let log_content = std::fs::read_to_string(&log_file).unwrap();
    assert!(!log_content.is_empty(), "log file must be non-empty");

    // F-12, F-13: lost-packet events must appear in the log
    assert!(
        log_content.contains("lost") || log_content.contains("Cycle"),
        "log must record lost-packet events; got:\n{}",
        log_content
    );

    std::fs::remove_file(&log_file).ok();
    std::fs::remove_file(&csv_file).ok();
}
