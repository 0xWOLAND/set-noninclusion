#![no_main]
sp1_zkvm::entrypoint!(main);

use sp1_verifier::Groth16Verifier;

fn main() {
    let n = sp1_zkvm::io::read::<u32>();

    let groth16_vk = *sp1_verifier::GROTH16_VK_BYTES;
    let sp1_vkey_hash: String = sp1_zkvm::io::read();

    let proof_results = (0..n)
        .map(|_| {
            let proof = sp1_zkvm::io::read_vec();
            let public_inputs = sp1_zkvm::io::read_vec();
            Groth16Verifier::verify(&proof, &public_inputs, &sp1_vkey_hash, groth16_vk)
        })
        .collect::<Vec<_>>();

    assert!(proof_results.iter().all(|result| result.is_ok()));
}
