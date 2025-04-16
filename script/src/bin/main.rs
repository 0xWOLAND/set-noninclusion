use std::path::Path;

use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use substrate_bn::{AffineG1, Fq, Fr};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const NONINCLUSION_ELF: &[u8] = include_elf!("noninclusion-program");

fn run_epoch(
    a_prev: AffineG1,
    s_prev: AffineG1,
    n: u32,
    v: Fr,
    r: Fr,
    roots: Vec<Fr>,
    epoch: u32,
) -> (AffineG1, AffineG1) {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    let mut a_prev_bytes = vec![0u8; 64];
    a_prev.x().to_big_endian(&mut a_prev_bytes[..32]).unwrap();
    a_prev.y().to_big_endian(&mut a_prev_bytes[32..64]).unwrap();

    let mut s_prev_bytes = vec![0u8; 64];
    s_prev.x().to_big_endian(&mut s_prev_bytes[..32]).unwrap();
    s_prev.y().to_big_endian(&mut s_prev_bytes[32..64]).unwrap();

    stdin.write(&n);
    stdin.write_slice(&a_prev_bytes);
    stdin.write_slice(&s_prev_bytes);

    // Roots
    roots.iter().for_each(|root| {
        let mut bytes = [0u8; 32];
        root.to_big_endian(&mut bytes).unwrap();
        stdin.write_slice(&bytes);
    });

    let mut v_bytes = [0u8; 32];
    v.to_big_endian(&mut v_bytes).unwrap();

    let mut r_bytes = [0u8; 32];
    r.to_big_endian(&mut r_bytes).unwrap();

    stdin.write_slice(&v_bytes);
    stdin.write_slice(&r_bytes);

    // Setup the program for proving.
    let (pk, vk) = client.setup(NONINCLUSION_ELF);

    let (mut output, _) = client.execute(NONINCLUSION_ELF, &stdin).run().unwrap();

    let mut s_next_bytes = vec![0u8; 64];
    output.read_slice(&mut s_next_bytes);

    let s_next_x = Fq::from_slice(&s_next_bytes[..32]).unwrap();
    let s_next_y = Fq::from_slice(&s_next_bytes[32..64]).unwrap();
    let s_next = AffineG1::new(s_next_x, s_next_y).unwrap();

    let mut a_next_bytes = vec![0u8; 64];
    output.read_slice(&mut a_next_bytes);

    let a_next_x = Fq::from_slice(&a_next_bytes[..32]).unwrap();
    let a_next_y = Fq::from_slice(&a_next_bytes[32..64]).unwrap();
    let a_next = AffineG1::new(a_next_x, a_next_y).unwrap();

    // Generate the proof
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("failed to generate proof");

    proof
        .save(Path::new(&format!("../proofs/proof_{}.bin", epoch)))
        .unwrap_or_else(|_| panic!("failed to save proof for epoch {}", epoch));

    (a_next, s_next)
}

fn main() {
    let mut a_prev = AffineG1::default();
    let mut s_prev = AffineG1::default();

    let n = 3;

    let v = Fr::from_str("100").unwrap();
    let r = Fr::from_str("420").unwrap();

    for epoch in 1..3 {
        let roots = (1..=n)
            .map(|i| Fr::from_str(&(epoch * n + i).to_string()).unwrap())
            .collect::<Vec<_>>();

        let (a_next, s_next) = run_epoch(a_prev, s_prev, n, v, r, roots, epoch);

        a_prev = a_next;
        s_prev = s_next;
    }
}
