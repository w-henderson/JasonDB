use std::error::Error;

pub fn quiet_assert(condition: bool) -> Result<(), Box<dyn Error>> {
    if condition {
        Ok(())
    } else {
        Err("Assertion failed".into())
    }
}
