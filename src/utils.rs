use crate::errors::{UserError, HandlerError};
use std::collections::HashMap;

pub fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}

/// take status=tagged&test=true and return dictionary of key-value
pub fn query_string_to_queries(s: &String) 
-> Result<HashMap<String, String>, HandlerError> {
    // TODO: implement this
    let mut dict = HashMap::new();
    dict.insert(
        "status".to_string(),
        "tagged".to_string(),
    );
    return Ok(dict);
}