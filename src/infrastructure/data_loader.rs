use std::{fs::File, io::Read};

use serde_json::Value;

pub fn load_jsonfiles_by_ids(ids: Vec<u64>) -> Result<Value, String> {
    let mut data = serde_json::Map::new();

    for id in &ids {
        let parsed_data = load_json_file(id)?;

        data.insert(id.clone().to_string(), parsed_data);
    }

    Ok(Value::Object(data))
}

fn load_json_file(id: &u64) -> Result<Value, String> {
    let path = format!("data/{}.json", id);

    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut data = String::new();

    file.read_to_string(&mut data).map_err(|e| e.to_string())?;

    let parsed_data: Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    Ok(parsed_data)
}
