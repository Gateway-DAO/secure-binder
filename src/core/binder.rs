use crate::core::loader;
use ark_bls12_381::G1Affine; // BLS12-381 affine group
use ark_ec::AffineRepr;
use ark_std::rand::RngCore;
use oblivious_transfer_protocols::base_ot::naor_pinkas_ot::{OTReceiver, OTSenderSetup};
use oblivious_transfer_protocols::configs::OTConfig;
use rand::rngs::OsRng;
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

            prepare_ot(selected_data);

            Ok(())
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}

fn prepare_ot(selected_data: Vec<Value>) {
    println!("Preparing OT for selected data: {:?}", selected_data);

    // Random number generator for OT
    let mut rng = OsRng;

    // Number of OT transfers and number of messages per transfer
    let num_ot = selected_data.len(); // Number of oblivious transfers
    let num_messages = 1; // Number of possible messages (1-of-2 OT)

    // Create the OT configuration object (set the number of OT and number of messages per OT)
    let config = OTConfig {
        num_ot: num_ot as u16, // Number of oblivious transfers
        num_messages,          // 1-of-2 OT (two possible messages in each transfer)
    };

    // Use the generator point of the BLS12-381 elliptic curve
    let g = G1Affine::generator();

    // OT Sender Setup: Initialize OT sender with configuration and public key
    let (sender, sender_pub_key) = OTSenderSetup::new(&mut rng, config.clone(), &g);

    // Choices for OT (e.g., the indices of the values the receiver wants to select)
    let choices = vec![0; num_ot]; // Example choices (0 for simplicity, adjust as necessary)

    // OT Receiver Setup: Initialize OT receiver with configuration, choices, and public key
    let (mut receiver, receiver_pub_key) =
        OTReceiver::new(&mut rng, config, choices, sender_pub_key.clone(), &g).unwrap();

    // Prepare and execute OT for each entry in the selected data
    for (index, value) in selected_data.iter().enumerate() {
        // Convert the value to a primitive (if applicable, e.g., integer)
        if let Some(int_value) = value.as_i64() {
            // Create two distinct messages for the 1-of-2 OT
            let message_1: Vec<u8> = vec![int_value as u8; 32]; // Example message 1: repeating the integer value as a byte array
            let message_2: Vec<u8> = vec![0; 32]; // Example message 2: a placeholder with zeros

            // Provide exactly two messages per OT
            let messages = vec![message_1, message_2];

            // Wrap the messages into a batch (Vec<Vec<Vec<u8>>>)
            let message_batch = vec![messages]; // This wraps the two messages into a batch

            // Sender encrypts the value for OT using the public key
            let sender_encryption = sender
                .encrypt(&mut rng, receiver_pub_key.clone(), message_batch)
                .unwrap();

            // Receiver decrypts the value via OT
            let receiver_value = receiver.decrypt(sender_encryption, index as u32).unwrap();

            println!("OT prepared: Receiver selected value: {:?}", receiver_value);
        } else {
            println!("Skipping non-integer value for OT preparation: {:?}", value);
        }
    }
}
