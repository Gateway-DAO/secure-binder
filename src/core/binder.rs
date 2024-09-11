use crate::core::loader;

use serde_json::Value;

pub fn run_binder(ids: Vec<u64>, param: String) -> Result<(), String> {
    match loader::load_data(ids) {
        Ok(data) => {
            let mut selected_data = Vec::new();
            for (id, entry) in data.as_object().unwrap().iter() {
                if let Some(value) = entry.get(&param) {
                    println!("ID: {}, {}: {}", id, param, value);
                    selected_data.push(value.clone());
                } else {
                    println!("Param '{}' not found for ID: {}", param, id);
                }
            }

            Ok(())
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}
