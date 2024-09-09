use crate::infrastructure::data_loader;

pub fn load_data() -> Result<(), String> {
    match data_loader::load_json_file() {
        Ok(data) => {
            println!("Data loaded successfully: {}", data);
            Ok(())
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}
