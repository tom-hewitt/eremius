use std::fs;

extern crate test;
use test::{black_box, Bencher};

use super::Emulator;

#[bench]
fn test_large_file(b: &mut Bencher) {
    let instructions = "ADD R0, R1, R2\n".repeat(100000);

    fs::write("./repeated.a", &instructions).unwrap();

    b.iter(|| {
        let data = fs::read_to_string("./repeated.a").unwrap();

        let mut emulator = Emulator::new();

        emulator.assemble(&data).unwrap();

        black_box(emulator);
    });
}

#[bench]
fn test_large_file_with_labels(b: &mut Bencher) {
    let instructions: String = (0..1000).map(|i| format!("l{} ADD R0, R1, R2\n", i)).collect();

    fs::write("./repeated.a", &instructions).unwrap();

    b.iter(|| {
        let data = fs::read_to_string("./repeated.a").unwrap();

        let mut emulator = Emulator::new();

        emulator.assemble(&data).unwrap();

        black_box(emulator);
    });
}

#[bench]
fn test_large_file_with_pseudo_instructions(b: &mut Bencher) {
    let instructions: String = (0..1000).map(|i| format!("l{} ADRL R0, l{}\n", i, i)).collect();

    fs::write("./repeated.a", &instructions).unwrap();

    b.iter(|| {
        let data = fs::read_to_string("./repeated.a").unwrap();

        let mut emulator = Emulator::new();

        emulator.assemble(&data).unwrap();

        black_box(emulator);
    });
}