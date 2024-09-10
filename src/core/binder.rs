use crate::core::loader;

pub fn run_binder(ids: Vec<u64>, param: String) -> Result<(), String> {
    match loader::load_data(ids) {
        Ok(data) => {
            for (id, entry) in data.as_object().unwrap().iter() {
                if let Some(value) = entry.get(&param) {
                    println!("ID: {}, {}: {}", id, param, value);
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
