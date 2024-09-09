use serde_json::Value;

use crate::infrastructure::data_loader;

pub fn load_data() -> Result<Value, String> {
    match data_loader::load_json_file() {
        Ok(data) => {
            println!("Data loaded successfully: {}", data);
            Ok(data)
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}
