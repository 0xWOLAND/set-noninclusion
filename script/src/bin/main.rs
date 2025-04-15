use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use substrate_bn::{AffineG1, Fr};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const NONINCLUSION_ELF: &[u8] = include_elf!("noninclusion-program");

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    let n = 3;

    let mut a_prev = AffineG1::default();
    let mut a_prev_bytes = vec![0u8; 64];
    a_prev.x().to_big_endian(&mut a_prev_bytes[..32]).unwrap();
    a_prev.y().to_big_endian(&mut a_prev_bytes[32..64]).unwrap();

    let mut s_prev = AffineG1::default();
    let mut s_prev_bytes = vec![0u8; 64];
    s_prev.x().to_big_endian(&mut s_prev_bytes[..32]).unwrap();
    s_prev.y().to_big_endian(&mut s_prev_bytes[32..64]).unwrap();

    stdin.write(&n);
    stdin.write_slice(&a_prev_bytes);
    stdin.write_slice(&s_prev_bytes);

    // Roots
    (1..=n).for_each(|i| {
        let num = Fr::from_str(&i.to_string()).unwrap();
        let mut bytes = [0u8; 32];
        num.to_big_endian(&mut bytes).unwrap();
        stdin.write_slice(&bytes);
    });

    let v = Fr::from_str("100").unwrap();
    let mut v_bytes = [0u8; 32];
    v.to_big_endian(&mut v_bytes).unwrap();

    let r = Fr::from_str("1").unwrap();
    let mut r_bytes = [0u8; 32];
    r.to_big_endian(&mut r_bytes).unwrap();

    stdin.write_slice(&v_bytes);
    stdin.write_slice(&r_bytes);

    // Setup the program for proving.
    let (pk, vk) = client.setup(NONINCLUSION_ELF);

    client.execute(NONINCLUSION_ELF, &stdin).run().unwrap();

    // // Generate the proof
    // let proof = client
    //     .prove(&pk, &stdin)
    //     .run()
    //     .expect("failed to generate proof");

    // println!("Successfully generated proof!");

    // // Verify the proof.
    // client.verify(&proof, &vk).expect("failed to verify proof");
    // println!("Successfully verified proof!");
}
