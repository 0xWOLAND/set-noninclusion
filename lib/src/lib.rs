use alloy_sol_types::sol;
use rand::thread_rng;
use sha2::{Digest, Sha256};
use sp1_hash2curve::{commit, HashToCurve};
use substrate_bn::{AffineG1, Fq, Fr, G1};

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 n;
        uint32 a;
        uint32 b;
    }
}

/// Compute the n'th fibonacci number (wrapping around on overflows), using normal Rust code.
pub fn fibonacci(n: u32) -> (u32, u32) {
    let mut a = 0u32;
    let mut b = 1u32;
    for _ in 0..n {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    (a, b)
}

fn vieta(vs: Vec<Fr>) -> Vec<Fr> {
    vs.iter().fold(vec![Fr::one()], |coeffs, &v| {
        let mut new_coeffs = vec![Fr::zero(); coeffs.len() + 1];

        new_coeffs
            .iter_mut()
            .skip(1)
            .zip(&coeffs)
            .for_each(|(new, &old)| *new = old);

        new_coeffs
            .iter_mut()
            .zip(&coeffs)
            .for_each(|(new, &old)| *new = *new - v * old);

        new_coeffs
    })
}

// Evaluate a polynomial given its roots
fn eval(roots: Vec<Fr>, x: Fr) -> Fr {
    roots.iter().fold(Fr::one(), |acc, &root| acc * (x - root))
}

pub fn insert(roots: Vec<Fr>, a_prev: AffineG1, r: Fr) -> AffineG1 {
    let coeffs = vieta(roots);
    let p_i = commit(&coeffs, AffineG1::default(), r);

    let mut p_i_bytes = [0u8; 64];
    p_i.x()
        .to_big_endian(&mut p_i_bytes[..32])
        .expect("Failed to convert x to big endian");
    p_i.y()
        .to_big_endian(&mut p_i_bytes[32..])
        .expect("Failed to convert y to big endian");

    let mut a_prev_bytes = [0u8; 64];
    a_prev
        .x()
        .to_big_endian(&mut a_prev_bytes[..32])
        .expect("Failed to convert x to big endian");
    a_prev
        .y()
        .to_big_endian(&mut a_prev_bytes[32..])
        .expect("Failed to convert y to big endian");

    let h = Sha256::new()
        .chain_update(p_i_bytes)
        .chain_update(a_prev_bytes)
        .finalize();

    let h = Fr::from_bytes_be_mod_order(&h[..]).expect("Failed to convert h to Fr");

    a_prev * h + p_i
}

pub fn check_non_membership(roots: Vec<Fr>, v: Fr, r: Fr, s_prev: AffineG1) -> AffineG1 {
    let alpha_i = eval(roots.clone(), v);

    assert!(!alpha_i.is_zero());

    let coeffs = vieta(roots);
    let p_i = commit(&coeffs, AffineG1::default(), r);
    let pp_i = p_i - AffineG1::default() * alpha_i;

    let mut s_prev_bytes = [0u8; 64];
    s_prev
        .x()
        .to_big_endian(&mut s_prev_bytes[..32])
        .expect("Failed to convert x to big endian");
    s_prev
        .y()
        .to_big_endian(&mut s_prev_bytes[32..])
        .expect("Failed to convert y to big endian");

    let mut pp_i_bytes = [0u8; 64];
    pp_i.x()
        .to_big_endian(&mut pp_i_bytes[..32])
        .expect("Failed to convert x to big endian");
    pp_i.y()
        .to_big_endian(&mut pp_i_bytes[32..])
        .expect("Failed to convert y to big endian");

    let hh_i = Sha256::new()
        .chain_update(s_prev_bytes)
        .chain_update(pp_i_bytes)
        .finalize();

    let hh_i = Fr::from_bytes_be_mod_order(&hh_i[..]).expect("Failed to convert hh_i to Fr");

    s_prev * hh_i + pp_i
}

#[test]
fn test_vieta() {
    // Test with a simple case: roots at 1 and 2
    // This should give us coefficients for (x-1)(x-2) = x^2 - 3x + 2
    let one = Fr::from_slice(&[1u8]).unwrap();
    let two = Fr::from_slice(&[2u8]).unwrap();
    let three = Fr::from_slice(&[3u8]).unwrap();
    let roots = vec![one, two];
    let coeffs = vieta(roots);

    // The coefficients should be [2, -3, 1] (constant term, linear term, quadratic term)
    assert_eq!(coeffs[0], two); // constant term
    assert_eq!(coeffs[1], -three); // linear term
    assert_eq!(coeffs[2], one); // quadratic term
}

#[test]
fn test_insert() {
    let roots = vec![
        Fr::from_slice(&[1u8]).unwrap(),
        Fr::from_slice(&[2u8]).unwrap(),
    ];
    let a_prev = AffineG1::default();
    let s_prev = AffineG1::default();

    let r = Fr::from_slice(&[3u8]).unwrap();
    let a_prev = insert(roots.clone(), a_prev, r);

    let v = Fr::from_slice(&[4u8]).unwrap();
    let s_prev = check_non_membership(roots, v, r, s_prev);

    println!("a_prev: {:?}", a_prev);
    println!("s_prev: {:?}", s_prev);

    let roots = vec![
        Fr::from_slice(&[3u8]).unwrap(),
        Fr::from_slice(&[4u8]).unwrap(),
    ];
    let v = Fr::from_slice(&[5u8]).unwrap();
    let a_prev = insert(roots.clone(), a_prev, r);
    let s_prev = check_non_membership(roots, v, r, s_prev);

    println!("a_prev: {:?}", a_prev);
    println!("s_prev: {:?}", s_prev);
}
