use clap::{Parser, Subcommand};
use drs_rt::echo;
use drs_rt::master::{self, MasterConfig};

// F-2: single binary, runtime role selection via CLI subcommand
#[derive(Parser)]
#[command(name = "drs-rt", about = "UDP round-trip time measurement tool")]
struct Cli {
    #[command(subcommand)]
    role: Role,
}

#[derive(Subcommand)]
enum Role {
    // F-1: Master role
    Master {
        // F-4: Echo node IP address (required)
        #[arg(long)]
        host: String,
        // F-3: UDP port, default 5000
        #[arg(long, default_value_t = 5000)]
        port: u16,
        // F-5: measurement cycle count (master only)
        #[arg(long, default_value_t = 50_000)]
        cycles: u64,
        // F-11: receive timeout in seconds
        #[arg(long, default_value_t = 5.0)]
        timeout: f64,
        // F-18: unrecorded warm-up cycles before measurement
        #[arg(long, default_value_t = 10)]
        warmup: u64,
        // F-19: optional CPU core to pin the measurement thread
        #[arg(long)]
        cpu_pin: Option<usize>,
        // F-14: CSV output file path
        #[arg(long, default_value = "rtt_results.csv")]
        output: String,
        // F-13: log file path
        #[arg(long, default_value = "drs_master.log")]
        log: String,
    },
    // F-1: Echo role
    Echo {
        // F-3: UDP port, default 5000
        #[arg(long, default_value_t = 5000)]
        port: u16,
        // F-13: log file path
        #[arg(long, default_value = "drs_echo.log")]
        log: String,
    },
}

fn setup_logger(log_path: &str) -> anyhow::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_path)?)
        .apply()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logger: {}", e))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.role {
        Role::Master {
            host,
            port,
            cycles,
            timeout,
            warmup,
            cpu_pin,
            output,
            log,
        } => {
            setup_logger(&log)?;
            master::run(MasterConfig {
                host,
                port,
                cycles,
                timeout_secs: timeout,
                warmup,
                cpu_pin,
                output,
            })
        }
        Role::Echo { port, log } => {
            setup_logger(&log)?;
            echo::run(port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // UT-1: valid CLI combinations parse without error
    #[test]
    fn cli_master_minimal() {
        Cli::parse_from(["drs-rt", "master", "--host", "192.168.1.42"]).role;
    }

    #[test]
    fn cli_master_all_args() {
        Cli::parse_from([
            "drs-rt", "master",
            "--host", "192.168.1.42",
            "--port", "9000",
            "--cycles", "100000",
            "--timeout", "2.0",
            "--warmup", "20",
            "--cpu-pin", "1",
            "--output", "out.csv",
            "--log", "master.log",
        ]);
    }

    #[test]
    fn cli_echo_defaults() {
        Cli::parse_from(["drs-rt", "echo"]);
    }

    #[test]
    fn cli_echo_custom_port() {
        Cli::parse_from(["drs-rt", "echo", "--port", "9001"]);
    }

    // UT-1: master requires --host; missing it is an error
    #[test]
    fn cli_master_missing_host_fails() {
        assert!(Cli::try_parse_from(["drs-rt", "master"]).is_err());
    }

    // UT-1: echo does not accept --host (enforced by subcommand structure)
    #[test]
    fn cli_echo_rejects_host() {
        assert!(Cli::try_parse_from(["drs-rt", "echo", "--host", "1.2.3.4"]).is_err());
    }

    // UT-1: echo does not accept --cycles (enforced by subcommand structure)
    #[test]
    fn cli_echo_rejects_cycles() {
        assert!(Cli::try_parse_from(["drs-rt", "echo", "--cycles", "1000"]).is_err());
    }
}
