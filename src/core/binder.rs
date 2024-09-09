use crate::core::loader;

pub fn run_binder() -> Result<(), String> {
    match loader::load_data() {
        Ok(data) => {
            if let Some(age) = data.get("age").and_then(|v| v.as_u64()) {
                println!("Age: {}", age);
            } else {
                println!("Age not found");
            }

            Ok(())
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}
