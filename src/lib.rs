use itertools::{izip, Itertools};
use p3_baby_bear::{BabyBear, DiffusionMatrixBabyBear};
use p3_challenger::{CanObserve, DuplexChallenger, FieldChallenger};
use p3_commit::{ExtensionMmcs, Pcs, PolynomialSpace};
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::{ExtensionField, Field, PackedValue};
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_matrix::dense::{DenseMatrix, RowMajorMatrix};
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon2::{Poseidon2, Poseidon2ExternalMatrixGeneral};
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_field::AbstractField;
use p3_matrix::Matrix;
use rand::distributions::{Distribution, Standard};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn seeded_rng() -> impl Rng {
    ChaCha20Rng::seed_from_u64(0)
}

fn do_test_fri_pcs<Val, Challenge, Challenger, P>(
    (pcs, challenger): &(P, Challenger),
    polynomial: RowMajorMatrix<Val>,
) where
    P: Pcs<Challenge, Challenger>,
    P::Domain: PolynomialSpace<Val = Val>,
    Val: Field,
    Standard: Distribution<Val>,
    Challenge: ExtensionField<Val>,
    Challenger: Clone + CanObserve<P::Commitment> + FieldChallenger<Val>,
{
    let mut p_challenger = challenger.clone();
    
    // Create domain for our polynomial
    let degree = polynomial.height();
    let domain = pcs.natural_domain_for_degree(degree);
    let domains_and_polys = vec![(domain, polynomial)];

    // Commit to the polynomial
    let (commit, data) = pcs.commit(domains_and_polys.clone());
    p_challenger.observe_slice(&[commit.clone()]);
    
    // Sample a random point for evaluation
    let zeta: Challenge = p_challenger.sample_ext_element();
    let points = vec![vec![zeta]];
    
    // Open the polynomial at zeta
    let data_and_points = vec![(&data, points)];
    let (opening, proof) = pcs.open(data_and_points, &mut p_challenger);

    // Verify the opening
    let mut v_challenger = challenger.clone();
    v_challenger.observe_slice(&[commit.clone()]);
    let verifier_zeta: Challenge = v_challenger.sample_ext_element();
    assert_eq!(verifier_zeta, zeta);

    let claims = vec![(domain, vec![(zeta, opening[0][0][0].clone())])];
    let commits_and_claims = vec![(commit, claims)];

    pcs.verify(commits_and_claims, &proof, &mut v_challenger)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define types
    type Val = BabyBear;
    type Challenge = BinomialExtensionField<Val, 4>;
    type Perm = Poseidon2<Val, Poseidon2ExternalMatrixGeneral, DiffusionMatrixBabyBear, 16, 7>;
    type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;
    type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;
    type ValMmcs = FieldMerkleTreeMmcs<Val, Val, MyHash, MyCompress, 8>;
    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    type Dft = Radix2DitParallel;
    type Challenger = DuplexChallenger<Val, Perm, 16, 8>;
    type MyPcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;

    fn setup_pcs() -> (MyPcs, Challenger) {
        let mut rng = seeded_rng();
        let perm = Perm::new_from_rng_128(
            Poseidon2ExternalMatrixGeneral,
            DiffusionMatrixBabyBear::default(),
            &mut rng,
        );
        let hash = MyHash::new(perm.clone());
        let compress = MyCompress::new(perm.clone());
        let val_mmcs = ValMmcs::new(hash, compress);
        let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
        let fri_config = FriConfig {
            log_blowup: 1,
            num_queries: 10,
            proof_of_work_bits: 8,
            mmcs: challenge_mmcs,
        };
        let pcs = MyPcs::new(Dft {}, val_mmcs, fri_config);
        let challenger = Challenger::new(perm);
        (pcs, challenger)
    }

    #[test]
    fn test_fri_pcs_with_simple_polynomial() {
        let degree = 8;
        let width = 1;
        let values: Vec<Val> = (0..degree)
            .map(|i| Val::from_canonical_u32((i + 1) as u32))
            .collect();
        let polynomial = RowMajorMatrix::new(values, width);

        let (pcs, challenger) = setup_pcs();
        do_test_fri_pcs(&(pcs, challenger), polynomial);
    }
}
