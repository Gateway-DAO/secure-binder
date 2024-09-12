use crate::core::loader;
use fancy_garbling::FancyInput;
use fancy_garbling::{
    twopac::semihonest::{Evaluator as FancyEvaluator, Garbler as FancyGarbler},
    util, AllWire, BinaryBundle, BinaryGadgets, Fancy, FancyArithmetic, FancyBinary, FancyReveal,
};
use ocelot::ot::{AlszReceiver as OtReceiver, AlszSender as OtSender};
use scuttlebutt::AesRng;
use scuttlebutt::{AbstractChannel, Channel};
use std::io::BufReader;

use std::{io::BufWriter, os::unix::net::UnixStream};

pub fn run_binder(ids: Vec<u64>, param: String) -> Result<(), String> {
    match loader::load_data(ids) {
        Ok(data) => {
            let mut selected_data = Vec::new();
            for (id, entry) in data.as_object().unwrap().iter() {
                if let Some(value) = entry.get(&param) {
                    println!("ID: {}, {}: {}", id, param, value);

                    // Try converting the serde_json::Value into u16
                    if let Some(num) = value.as_u64() {
                        if num <= u16::MAX as u64 {
                            selected_data.push(num as u16); // Convert to u16
                        } else {
                            println!("Value {} is too large for u16, skipping", num);
                        }
                    } else {
                        println!("Value for ID {} is not a number, skipping", id);
                    }
                } else {
                    println!("Param '{}' not found for ID: {}", param, id);
                }
            }

            println!("Selected data: {:?}", selected_data);

            if selected_data.is_empty() {
                return Err("No data found for the given param".into());
            }

            // Call the fancy garbling summation with the selected_data
            let result = garbled_sum(selected_data)?;
            println!("Garbled Circuit Sum Result: {}", result);

            Ok(())
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}

struct SUMInputs<F> {
    pub garbler_wires: BinaryBundle<F>,
    pub evaluator_wires: BinaryBundle<F>,
}

fn garbled_sum(selected_data: Vec<u16>) -> Result<u128, String> {
    let (sender, receiver) = UnixStream::pair().unwrap();

    let garbler_input: u128 = selected_data.iter().map(|&x| x as u128).sum(); // Summing for the garbler
    let evaluator_input: u128 = 0; // For simplicity, the evaluator can be 0

    // Spawn a thread for the garbler
    std::thread::spawn(move || {
        let rng_gb = AesRng::new();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let mut channel = Channel::new(reader, writer);
        gb_sum(&mut rng_gb.clone(), &mut channel, garbler_input);
    });

    // The evaluator runs on the main thread
    let rng_ev = AesRng::new();
    let reader = BufReader::new(receiver.try_clone().unwrap());
    let writer = BufWriter::new(receiver);
    let mut channel = Channel::new(reader, writer);

    // Sum in clear for verification (optional)
    let expected_sum = selected_data.iter().map(|&x| x as u128).sum::<u128>();
    let result = ev_sum(&mut rng_ev.clone(), &mut channel, evaluator_input);

    println!("Expected sum in clear: {}", expected_sum);
    assert!(
        result == expected_sum,
        "The garbled circuit result is incorrect"
    );

    Ok(result)
}

fn gb_sum<C>(rng: &mut AesRng, channel: &mut C, input: u128)
where
    C: AbstractChannel + std::clone::Clone,
{
    let mut gb =
        FancyGarbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone()).unwrap();
    let circuit_wires = gb_set_fancy_inputs(&mut gb, input);

    // Debug: Display garbler wire labels
    println!(
        "Garbler wire labels (obfuscated): {:?}",
        circuit_wires.garbler_wires.wires()
    );

    let sum =
        fancy_sum::<FancyGarbler<C, AesRng, OtSender, AllWire>>(&mut gb, circuit_wires).unwrap();

    // Debug: Display sum wire labels before output
    println!("Sum wire labels (garbler side): {:?}", sum.wires());

    gb.outputs(sum.wires()).unwrap();
}

fn ev_sum<C>(rng: &mut AesRng, channel: &mut C, input: u128) -> u128
where
    C: AbstractChannel + std::clone::Clone,
{
    let mut ev =
        FancyEvaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone())
            .unwrap();
    let circuit_wires = ev_set_fancy_inputs(&mut ev, input);

    // Debug: Display evaluator wire labels
    println!(
        "Evaluator wire labels (obfuscated): {:?}",
        circuit_wires.evaluator_wires.wires()
    );

    let sum = fancy_sum::<FancyEvaluator<C, AesRng, OtReceiver, AllWire>>(&mut ev, circuit_wires)
        .unwrap();

    let sum_binary = ev
        .outputs(sum.wires())
        .unwrap()
        .expect("evaluator should produce outputs");

    // Debug: Output sum in binary form
    println!("Sum in binary (evaluator side): {:?}", sum_binary);

    util::u128_from_bits(&sum_binary)
}

fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: u128) -> SUMInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: std::fmt::Debug,
{
    let nbits = 128;
    let garbler_wires: BinaryBundle<F::Item> = gb.bin_encode(input, nbits).unwrap();
    let evaluator_wires: BinaryBundle<F::Item> = gb.bin_receive(nbits).unwrap();

    // Debug: Print garbler wire labels (obfuscated)
    println!("Garbler wire labels: {:?}", garbler_wires.wires());

    SUMInputs {
        garbler_wires,
        evaluator_wires,
    }
}

fn ev_set_fancy_inputs<F, E>(ev: &mut F, input: u128) -> SUMInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: std::fmt::Debug,
{
    let nbits = 128;
    let garbler_wires: BinaryBundle<F::Item> = ev.bin_receive(nbits).unwrap();
    let evaluator_wires: BinaryBundle<F::Item> = ev.bin_encode(input, nbits).unwrap();

    // Debug: Print evaluator wire labels (obfuscated)
    println!("Evaluator wire labels: {:?}", evaluator_wires.wires());

    SUMInputs {
        garbler_wires,
        evaluator_wires,
    }
}

fn fancy_sum<F>(
    f: &mut F,
    wire_inputs: SUMInputs<F::Item>,
) -> Result<BinaryBundle<F::Item>, F::Error>
where
    F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
{
    let sum = f.bin_addition_no_carry(&wire_inputs.garbler_wires, &wire_inputs.evaluator_wires)?;

    Ok(sum)
}
