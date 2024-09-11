mod core;
mod infrastructure;
mod interfaces;

use crate::core::binder;

fn main() {
    let test_ids = vec![1, 2];
    let test_param = "age".to_string();

    match binder::run_binder(test_ids, test_param) {
        Ok(_) => println!("Binder ran successfully"),
        Err(e) => println!("Error running binder: {}", e),
    }
}

//TODO: Translate to OT
