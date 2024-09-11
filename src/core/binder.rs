use crate::core::loader;

use fancy_garbling::{
    circuit::{BinaryCircuit as Circuit, EvaluableCircuit},
    twopac::semihonest::{Evaluator, Garbler},
    FancyInput, WireMod2,
};
use ocelot::ot::{AlszReceiver as OtReceiver, AlszSender as OtSender};
use scuttlebutt::{unix_channel_pair, AesRng, UnixChannel};
use serde_json::Value;
use std::{fs::File, io::BufReader, time::SystemTime};

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
                            // Ensure the value fits within u16
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

            println!("It is an {:?}", selected_data);

            let mut circ = circuit("circuits/adder_32bit.txt");

            if selected_data.is_empty() {
                return Err("No data found for the given param".into());
            }

            run_circuit(&mut circ, selected_data.clone(), vec![]);

            Ok(())
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            Err(e)
        }
    }
}

fn ot_create_and_send(
    gb_inputs: Vec<u16>,
    circ_: Circuit,
    sender: UnixChannel,
    n_ev_inputs: usize,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let rng = AesRng::new();
        let mut gb = Garbler::<UnixChannel, AesRng, OtSender, WireMod2>::new(sender, rng).unwrap();
        println!("Garbler :: Initialization complete.");

        // Encode inputs
        let xs = gb
            .encode_many(&gb_inputs, &vec![2; gb_inputs.len()])
            .unwrap();
        let ys = gb.receive_many(&vec![2; n_ev_inputs]).unwrap();

        // Garble the circuit
        circ_.eval(&mut gb, &xs, &ys).unwrap();
        println!("Garbler :: Circuit garbling complete.");
    })
}

fn circuit(fname: &str) -> Circuit {
    println!("* Loading Circuit: {}", fname);
    Circuit::parse(BufReader::new(File::open(fname).unwrap())).unwrap()
}

fn ot_receive(
    receiver: UnixChannel,
    ev_inputs: Vec<u16>,
    n_gb_inputs: usize,
) -> (Vec<WireMod2>, Vec<WireMod2>) {
    let rng = AesRng::new();
    let mut ev =
        Evaluator::<UnixChannel, AesRng, OtReceiver, WireMod2>::new(receiver, rng).unwrap();
    println!("Evaluator :: Initialization complete.");

    // Receive Garbler inputs
    let xs = ev.receive_many(&vec![2; n_gb_inputs]).unwrap();
    let ys = ev
        .encode_many(&ev_inputs, &vec![2; ev_inputs.len()])
        .unwrap();

    (xs, ys)
}

fn compute_circuit(
    circ: &mut Circuit,
    xs: Vec<WireMod2>,
    ys: Vec<WireMod2>,
    receiver: UnixChannel,
    rng: AesRng,
) {
    let mut evaluator =
        Evaluator::<UnixChannel, AesRng, OtReceiver, WireMod2>::new(receiver, rng).unwrap();
    circ.eval(&mut evaluator, &xs, &ys).unwrap();
    println!("Circuit evaluation complete.");
}

fn run_circuit(circ: &mut Circuit, gb_inputs: Vec<u16>, ev_inputs: Vec<u16>) {
    let circ_ = circ.clone();
    let (sender, receiver) = unix_channel_pair();
    let n_gb_inputs = gb_inputs.len();
    let n_ev_inputs = ev_inputs.len();

    // Step 1: Create and send OT
    let handle = ot_create_and_send(gb_inputs, circ_, sender, n_ev_inputs);

    // Step 2: Receive OT and initialize rng
    let rng = AesRng::new();
    let (xs, ys) = ot_receive(receiver.clone(), ev_inputs, n_gb_inputs);

    // Step 3: Compute the circuit
    compute_circuit(circ, xs, ys, receiver, rng);

    // Wait for the Garbler's thread to complete
    handle.join().unwrap();
}
