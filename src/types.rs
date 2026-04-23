// F-17: sample status values
#[derive(Clone, Copy)]
pub enum Status {
    Ok,
    Timeout,
    SeqMismatch,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Timeout => "timeout",
            Status::SeqMismatch => "seq_mismatch",
        }
    }
}

// F-9, F-17: one sample per measurement cycle
pub struct Sample {
    pub timestamp_us: u64,
    pub rtt_us: i64, // -1 for lost cycles
    pub status: Status,
}
