use std::{fs::File, io::Read};

use serde_json::Value;

pub fn load_json_file() -> Result<Value, String> {
    let path = "data/mock1.json";

    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut data = String::new();

    file.read_to_string(&mut data).map_err(|e| e.to_string())?;

    let parsed_data: Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    Ok(parsed_data)
}
