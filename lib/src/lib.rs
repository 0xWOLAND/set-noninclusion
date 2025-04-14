use sha2::{Digest, Sha256};
use sp1_hash2curve::commit;
use substrate_bn::{AffineG1, Fr};

fn vieta(roots: &[Fr]) -> Vec<Fr> {
    roots.iter().fold(vec![Fr::one()], |coeffs, &r| {
        let mut next = vec![Fr::zero(); coeffs.len() + 1];

        (0..coeffs.len()).for_each(|i| {
            next[i + 1] = coeffs[i];
            next[i] -= r * coeffs[i];
        });

        next
    })
}

fn eval_poly_at_roots(roots: &[Fr], x: Fr) -> Fr {
    roots.iter().fold(Fr::one(), |acc, &r| acc * (x - r))
}

pub fn insert(roots: &[Fr], a_prev: AffineG1, r: Fr) -> Result<AffineG1, String> {
    let coeffs = vieta(roots);
    let p_i = commit(&coeffs, AffineG1::default(), r);

    let h = hash_points(&p_i, &a_prev)?;
    Ok(a_prev * h + p_i)
}

pub fn check_non_membership(
    roots: &[Fr],
    v: Fr,
    r: Fr,
    s_prev: AffineG1,
) -> Result<AffineG1, String> {
    let alpha = eval_poly_at_roots(roots, v);
    if alpha.is_zero() {
        return Err("Value is a member of the inserted set.".into());
    }

    let coeffs = vieta(roots);
    let p_i = commit(&coeffs, AffineG1::default(), r);
    let p_prime = p_i - AffineG1::default() * alpha;

    let h = hash_points(&s_prev, &p_prime)?;
    Ok(s_prev * h + p_prime)
}

fn hash_points(p1: &AffineG1, p2: &AffineG1) -> Result<Fr, String> {
    let mut bytes = [0u8; 128];

    p1.x()
        .to_big_endian(&mut bytes[0..32])
        .map_err(|_| "Failed to encode x1")?;
    p1.y()
        .to_big_endian(&mut bytes[32..64])
        .map_err(|_| "Failed to encode y1")?;
    p2.x()
        .to_big_endian(&mut bytes[64..96])
        .map_err(|_| "Failed to encode x2")?;
    p2.y()
        .to_big_endian(&mut bytes[96..128])
        .map_err(|_| "Failed to encode y2")?;

    let hash = Sha256::digest(bytes);
    Fr::from_bytes_be_mod_order(&hash).map_err(|_| "Hash did not map to valid Fr element".into())
}

#[test]
fn test_dynamic_non_membership_chain() -> Result<(), String> {
    let mut a_prev = AffineG1::default();
    let mut s_prev = AffineG1::default();
    let r = Fr::from_slice(&[42u8]).map_err(|_| "invalid Fr")?;
    let v = Fr::from_slice(&[99u8]).map_err(|_| "invalid Fr")?;

    let steps = vec![[1u8, 2u8], [3u8, 4u8], [5u8, 6u8]];

    for bytes in steps {
        let roots: Vec<Fr> = bytes
            .iter()
            .map(|b| Fr::from_slice(&[*b]).map_err(|_| "invalid Fr"))
            .collect::<Result<_, _>>()?;

        a_prev = insert(&roots, a_prev, r)?;
        s_prev = check_non_membership(&roots, v, r, s_prev)?;
    }

    let bad_roots = vec![
        Fr::from_slice(&[99u8]).unwrap(),
        Fr::from_slice(&[2u8]).unwrap(),
    ];

    a_prev = insert(&bad_roots, a_prev, r)?;
    assert!(check_non_membership(&bad_roots, v, r, s_prev).is_err());

    Ok(())
}
