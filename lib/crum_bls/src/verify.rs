/// Verification of signatures and unmasking
use bls12_381::{Bls12, G1Affine, G2Affine};
use pairing::{
    MultiMillerLoop,
    group::{Curve, Group},
};

use crate::{
    hash_to_curve::hash_to_curve,
    types::{PublicKey, Signature},
};

pub fn verify(message: &[u8], pk: &PublicKey, sig: &Signature) -> bool {
    let h = hash_to_curve(message).to_affine();

    // e(sig, G1) * e(h, -PK) == 1
    // Using BLS12-381 standard pairing check
    let is_valid = Bls12::multi_miller_loop(&[
        (&G1Affine::from(*sig), &G2Affine::generator().into()),
        (&G1Affine::from(h), &(-G2Affine::from(*pk)).into()),
    ])
    .final_exponentiation()
    .is_identity();

    is_valid.into()
}

/// The "Audit" - Verifies a single card peeling step: e(U, G2) == e(M, PK)
pub fn verify_unmasking(m: G1Affine, u: G1Affine, pk: G2Affine) -> bool {
    let g2_gen = G2Affine::generator();
    Bls12::multi_miller_loop(&[
        (&u, &G2Affine::from(pk).into()),
        (&m, &(-G2Affine::from(g2_gen)).into()),
    ])
    .final_exponentiation()
    .is_identity()
    .into()
}
