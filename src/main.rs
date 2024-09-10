mod core;
mod infrastructure;
mod interfaces;

use crate::core::binder;

fn main() {
    match binder::run_binder() {
        Ok(_) => println!("Binder ran successfully"),
        Err(e) => println!("Error running binder: {}", e),
    }
}
