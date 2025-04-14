#![no_main]
sp1_zkvm::entrypoint!(main);

use fibonacci_lib::check_non_membership;
use substrate_bn::{AffineG1, Fq, Fr};

pub fn main() {
    let n = sp1_zkvm::io::read::<u32>();

    let a_prev_x = Fq::from_slice(&sp1_zkvm::io::read_vec())
        .map_err(|_| "invalid Fq")
        .unwrap();
    let a_prev_y = Fq::from_slice(&sp1_zkvm::io::read_vec())
        .map_err(|_| "invalid Fq")
        .unwrap();
    let a_prev = AffineG1::new(a_prev_x, a_prev_y).unwrap();

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

    let s_prev_x = Fq::from_slice(&sp1_zkvm::io::read_vec())
        .map_err(|_| "invalid Fq")
        .unwrap();
    let s_prev_y = Fq::from_slice(&sp1_zkvm::io::read_vec())
        .map_err(|_| "invalid Fq")
        .unwrap();

    let s_prev = AffineG1::new(s_prev_x, s_prev_y).unwrap();
    let s_next = check_non_membership(&roots, v, r, s_prev).unwrap();

    let mut s_next_x_bytes = vec![0u8; 32];
    let mut s_next_y_bytes = vec![0u8; 32];

    s_next.x().to_big_endian(&mut s_next_x_bytes).unwrap();
    s_next.y().to_big_endian(&mut s_next_y_bytes).unwrap();

    sp1_zkvm::io::commit(&s_next_x_bytes);
    sp1_zkvm::io::commit(&s_next_y_bytes);

    let mut a_prev_x_bytes = vec![0u8; 32];
    let mut a_prev_y_bytes = vec![0u8; 32];

    a_prev.x().to_big_endian(&mut a_prev_x_bytes).unwrap();
    a_prev.y().to_big_endian(&mut a_prev_y_bytes).unwrap();

    sp1_zkvm::io::commit(&a_prev_x_bytes);
    sp1_zkvm::io::commit(&a_prev_y_bytes);
}
