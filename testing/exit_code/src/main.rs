use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(
        env::args()
            .skip(1)
            .next()
            .as_deref()
            .unwrap_or("1")
            .parse::<u8>()
            .unwrap_or(255)
    )
}
