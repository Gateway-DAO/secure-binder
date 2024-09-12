use crate::core::loader;
use fancy_garbling::FancyInput;
use fancy_garbling::{
    twopac::semihonest::{Evaluator as FancyEvaluator, Garbler as FancyGarbler},
    util, AllWire, BinaryBundle, BinaryGadgets, Fancy, FancyBinary, FancyReveal,
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

            // Call the fancy garbling comparison with the selected_data
            for value in selected_data {
                let result = garbled_compare(value as u128)?;
                println!("Is {} greater than or equal to 18? {}", value, result);
            }

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

fn garbled_compare(input: u128) -> Result<bool, String> {
    let (sender, receiver) = UnixStream::pair().unwrap();

    let garbler_input: u128 = input; // Input to be compared
    let evaluator_input: u128 = 18; // The constant to compare against (18)

    // Spawn a thread for the garbler
    std::thread::spawn(move || {
        let rng_gb = AesRng::new();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let mut channel = Channel::new(reader, writer);
        gb_compare(&mut rng_gb.clone(), &mut channel, garbler_input);
    });

    // The evaluator runs on the main thread
    let rng_ev = AesRng::new();
    let reader = BufReader::new(receiver.try_clone().unwrap());
    let writer = BufWriter::new(receiver);
    let mut channel = Channel::new(reader, writer);

    // Perform comparison in the garbled circuit
    let result = ev_compare(&mut rng_ev.clone(), &mut channel, evaluator_input);

    Ok(result)
}

fn gb_compare<C>(rng: &mut AesRng, channel: &mut C, input: u128)
where
    C: AbstractChannel + std::clone::Clone,
{
    let mut gb =
        FancyGarbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone()).unwrap();
    let circuit_wires = gb_set_fancy_inputs(&mut gb, input);

    // Perform greater-than-or-equal-to comparison using bin_geq
    let result = gb
        .bin_geq(&circuit_wires.garbler_wires, &circuit_wires.evaluator_wires)
        .unwrap();

    // Pass the result directly to `outputs` as it is a single wire
    gb.outputs(&[result]).unwrap();
}

fn ev_compare<C>(rng: &mut AesRng, channel: &mut C, input: u128) -> bool
where
    C: AbstractChannel + std::clone::Clone,
{
    let mut ev =
        FancyEvaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone())
            .unwrap();
    let circuit_wires = ev_set_fancy_inputs(&mut ev, input);

    // Perform greater-than-or-equal-to comparison using bin_geq
    let result = ev
        .bin_geq(&circuit_wires.garbler_wires, &circuit_wires.evaluator_wires)
        .unwrap();

    // Pass the result directly to `outputs` as it is a single wire
    let result_binary = ev
        .outputs(&[result])
        .unwrap()
        .expect("evaluator should produce outputs");

    // Interpret the result as boolean (true if x >= y, false otherwise)
    util::u128_from_bits(&result_binary) == 1
}

fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: u128) -> SUMInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: std::fmt::Debug,
{
    let nbits = 128;
    let garbler_wires: BinaryBundle<F::Item> = gb.bin_encode(input, nbits).unwrap();
    let evaluator_wires: BinaryBundle<F::Item> = gb.bin_receive(nbits).unwrap();

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

    SUMInputs {
        garbler_wires,
        evaluator_wires,
    }
}
