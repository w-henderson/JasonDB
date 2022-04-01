use crate::error::JasonError;

use humphrey_json::Value;

pub fn get_value(index: &str, json: &Value) -> Result<Value, JasonError> {
    let indexing_path = index.split('.');
    let mut current_json = json;
    for index in indexing_path {
        current_json = current_json.get(index).ok_or(JasonError::JsonError)?;
    }

    Ok(current_json.clone())
}

pub fn get_number(index: &str, json: &Value) -> Result<f64, JasonError> {
    let value = get_value(index, json)?;
    let number = value.as_number().ok_or(JasonError::JsonError)?;

    Ok(number)
}
