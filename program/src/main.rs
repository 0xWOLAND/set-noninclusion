#![no_main]
sp1_zkvm::entrypoint!(main);

use noninclusion_lib::{check_non_membership, insert};
use substrate_bn::{AffineG1, Fq, Fr};

pub fn main() {
    let n = sp1_zkvm::io::read::<u32>();

    let a_prev_bytes = sp1_zkvm::io::read_vec();
    let a_prev_x = Fq::from_slice(&a_prev_bytes[..32]).unwrap();
    let a_prev_y = Fq::from_slice(&a_prev_bytes[32..64]).unwrap();
    let a_prev = AffineG1::new(a_prev_x, a_prev_y).unwrap();

    let s_prev_bytes = sp1_zkvm::io::read_vec();
    let s_prev_x = Fq::from_slice(&s_prev_bytes[..32]).unwrap();
    let s_prev_y = Fq::from_slice(&s_prev_bytes[32..64]).unwrap();
    let s_prev = AffineG1::new(s_prev_x, s_prev_y).unwrap();

    let roots = (0..n)
        .map(|_| {
            let bytes = sp1_zkvm::io::read_vec();
            Fr::from_slice(&bytes).map_err(|_| "invalid Fr").unwrap()
        })
        .collect::<Vec<_>>();

    let v = Fr::from_slice(&sp1_zkvm::io::read_vec())
        .map_err(|_| "invalid Fr")
        .unwrap();

    let r = Fr::from_slice(&sp1_zkvm::io::read_vec())
        .map_err(|_| "invalid Fr")
        .unwrap();

    let s_next = check_non_membership(&roots, v, r, s_prev).unwrap();

    let mut s_next_bytes = vec![0u8; 64];
    s_next.x().to_big_endian(&mut s_next_bytes[..32]).unwrap();
    s_next.y().to_big_endian(&mut s_next_bytes[32..64]).unwrap();

    sp1_zkvm::io::commit_slice(&s_next_bytes);

    let mut a_next_bytes = vec![0u8; 64];
    let a_next = insert(&roots, a_prev, r).unwrap();

    a_next.x().to_big_endian(&mut a_next_bytes[..32]).unwrap();
    a_next.y().to_big_endian(&mut a_next_bytes[32..64]).unwrap();

    sp1_zkvm::io::commit_slice(&a_next_bytes);
}
