use crate::error::JasonError;

pub fn quiet_assert(condition: bool, e: JasonError) -> Result<(), JasonError> {
    if condition {
        Ok(())
    } else {
        Err(e)
    }
}
