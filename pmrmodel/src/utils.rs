use std::time::{
    SystemTime,
    SystemTimeError,
    UNIX_EPOCH
};

pub fn timestamp() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
