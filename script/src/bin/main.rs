use std::path::Path;

use sha2::{digest::Update, Digest, Sha256};
use sp1_sdk::{
    include_elf, EnvProver, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
};
use substrate_bn::{AffineG1, Fq, Fr};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const NONINCLUSION_ELF: &[u8] = include_elf!("noninclusion-program");
pub const FOLD_ELF: &[u8] = include_elf!("fold-program");

struct EpochParams {
    a_prev: AffineG1,
    s_prev: AffineG1,
    n: u32,
    v: Fr,
    r: Fr,
    roots: Vec<Fr>,
    epoch: u32,
}

impl EpochParams {
    fn hash(&self) -> [u8; 32] {
        let hasher = Sha256::new();

        let mut a_prev_bytes = [0u8; 64];
        self.a_prev
            .x()
            .to_big_endian(&mut a_prev_bytes[..32])
            .unwrap();
        self.a_prev
            .y()
            .to_big_endian(&mut a_prev_bytes[32..])
            .unwrap();

        let mut s_prev_bytes = [0u8; 64];
        self.s_prev
            .x()
            .to_big_endian(&mut s_prev_bytes[..32])
            .unwrap();
        self.s_prev
            .y()
            .to_big_endian(&mut s_prev_bytes[32..])
            .unwrap();

        let mut v_bytes = [0u8; 32];
        self.v.to_big_endian(&mut v_bytes).unwrap();

        let mut r_bytes = [0u8; 32];
        self.r.to_big_endian(&mut r_bytes).unwrap();

        let root_bytes_iter: Vec<u8> = self
            .roots
            .iter()
            .flat_map(|root| {
                let mut buf = [0u8; 32];
                root.to_big_endian(&mut buf).unwrap();
                buf
            })
            .collect();

        let result = hasher
            .chain_update(a_prev_bytes)
            .chain_update(s_prev_bytes)
            .chain_update(self.n.to_be_bytes())
            .chain_update(v_bytes)
            .chain_update(r_bytes)
            .chain_update(root_bytes_iter)
            .chain_update(self.epoch.to_be_bytes())
            .finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

fn run_epoch(
    params: EpochParams,
    client: &EnvProver,
    pk: &SP1ProvingKey,
) -> ((AffineG1, AffineG1), SP1ProofWithPublicValues) {
    let mut stdin = SP1Stdin::new();

    let mut a_prev_bytes = vec![0u8; 64];
    params
        .a_prev
        .x()
        .to_big_endian(&mut a_prev_bytes[..32])
        .unwrap();
    params
        .a_prev
        .y()
        .to_big_endian(&mut a_prev_bytes[32..64])
        .unwrap();

    let mut s_prev_bytes = vec![0u8; 64];
    params
        .s_prev
        .x()
        .to_big_endian(&mut s_prev_bytes[..32])
        .unwrap();
    params
        .s_prev
        .y()
        .to_big_endian(&mut s_prev_bytes[32..64])
        .unwrap();

    stdin.write(&params.n);
    stdin.write_slice(&a_prev_bytes);
    stdin.write_slice(&s_prev_bytes);

    // Roots
    params.roots.iter().for_each(|root| {
        let mut bytes = [0u8; 32];
        root.to_big_endian(&mut bytes).unwrap();
        stdin.write_slice(&bytes);
    });

    let mut v_bytes = [0u8; 32];
    params.v.to_big_endian(&mut v_bytes).unwrap();

    let mut r_bytes = [0u8; 32];
    params.r.to_big_endian(&mut r_bytes).unwrap();

    stdin.write_slice(&v_bytes);
    stdin.write_slice(&r_bytes);

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
    let path = format!(
        "../proofs/{}_ep{}.bin",
        hex::encode(params.hash()),
        params.epoch
    );
    let proof = if Path::new(&path).exists() {
        println!("Proof already exists");
        SP1ProofWithPublicValues::load(Path::new(&path)).expect("failed to load proof")
    } else {
        println!("Generating proof");
        let proof = client
            .prove(pk, &stdin)
            .compressed()
            .run()
            .expect("failed to generate proof");

        proof
            .save(Path::new(&path))
            .unwrap_or_else(|_| panic!("failed to save proof for epoch {}", params.epoch));
        proof
    };

    let accumulator = (a_next, s_next);

    (accumulator, proof)
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let client = ProverClient::from_env();
    let (pk, _) = client.setup(NONINCLUSION_ELF);

    let mut a_prev = AffineG1::default();
    let mut s_prev = AffineG1::default();

    let n = 3;
    let v = Fr::from_str("100").unwrap();
    let r = Fr::from_str("420").unwrap();

    let mut fold_proof = Vec::new();

    for epoch in 1..3 {
        let roots = (1..=n)
            .map(|i| Fr::from_str(&(epoch * n + i).to_string()).unwrap())
            .collect::<Vec<_>>();

        let params = EpochParams {
            a_prev,
            s_prev,
            n,
            v,
            r,
            roots,
            epoch,
        };

        let ((a_next, s_next), proof) = run_epoch(params, &client, &pk);

        a_prev = a_next;
        s_prev = s_next;

        fold_proof.push(proof);
    }

    let (pk, vk) = client.setup(FOLD_ELF);
    let mut stdin = SP1Stdin::new();

    stdin.write(&n);
    stdin.write(&vk);

    for proof in fold_proof {
        stdin.write_slice(&proof.bytes());
        stdin.write_slice(&proof.public_values.to_vec());
    }

    let (output, _) = client.execute(FOLD_ELF, &stdin).run().unwrap();
    println!("Proof done!");
}
