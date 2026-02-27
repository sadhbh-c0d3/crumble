//! Crumble (CRyptographic gaMBLE)
//! 
//! Mental Poker (1979) implemented using Boneh–Lynn–Shacham (BLS) cryptography.
//! Designed by the Sonia Code & Gemini AI (2026)
//! 
//! Copyright (c) 2026 Sonia Code; See LICENSE file for license details.

use std::collections::HashSet;

/// Verification of signatures and unmasking
use bls12_381::{Bls12, G1Affine, G2Affine, G2Prepared};
use pairing::{
    MultiMillerLoop,
    group::{Curve, Group},
};

use crate::{
    hash_to_curve::hash_to_curve,
    types::{PublicKey, Signature},
};

/// Verifies that message has been signed by signing key corresponding to public key.
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

/// Verifies that "masked" data has been "unmasked" with signing key
/// corresponding to public key.
pub fn verify_unmasking(masked: G1Affine, unmasked: G1Affine, pk: G2Affine) -> bool {
    let g2_gen = G2Affine::generator();
    Bls12::multi_miller_loop(&[
        (&unmasked, &G2Affine::from(pk).into()),
        (&masked, &(-G2Affine::from(g2_gen)).into()),
    ])
    .final_exponentiation()
    .is_identity()
    .into()
}

/// Verifies that "masked_before" data has been shuffled into "masked_after"
/// data with signing key corresponding to public key.
pub fn verify_shuffle(
    masked_before: &[G1Affine],
    masked_after: &[G1Affine],
    pk: &G2Affine,
) -> Result<(), &'static str> {
    if masked_before.len() < masked_after.len() {
        return Err("Masked before must at least same length as masked after");
    }

    let pk_prepared = G2Prepared::from(*pk);
    let neg_g2_gen = -G2Affine::generator();
    let neg_g2_prepared = G2Prepared::from(neg_g2_gen);

    let mut available_before = masked_before.to_vec();

    for point_after in masked_after {
        let mut matched_index = None;

        for (i, point_before) in available_before.iter().enumerate() {
            // e(card_after, -G2) * e(card_before, PK) == 1
            let is_match: bool = Bls12::multi_miller_loop(&[
                (point_after, &neg_g2_prepared),
                (point_before, &pk_prepared),
            ])
            .final_exponentiation()
            .is_identity()
            .into();

            if is_match {
                matched_index = Some(i);
                break;
            }
        }

        match matched_index {
            Some(idx) => {
                available_before.remove(idx);
            }
            None => {
                return Err("Cryptographic forgery detected");
            }
        }
    }

    Ok(())
}

pub struct ShuffleTrace {
    pub after_index: usize,
    pub claimed_before_index: usize,
}

pub fn verify_shuffle_traced(
    deck_before: &[G1Affine],
    deck_after: &[G1Affine],
    pk: &G2Affine,
    traces: &[ShuffleTrace], // Only M traces submitted
) -> Result<(), &'static str> {
    let pk_prepared = G2Prepared::from(*pk);
    let neg_g2_gen = -G2Affine::generator();
    let neg_g2_prepared = G2Prepared::from(neg_g2_gen);

    // 1. THE BIJECTION CHECK (Crucial for preventing cloned cards)
    let mut used_before_indices = HashSet::new();

    for trace in traces {
        // Prevent out-of-bounds panics
        if trace.after_index >= deck_after.len() || trace.claimed_before_index >= deck_before.len()
        {
            return Err("Trace index out of bounds");
        }

        // Ensure no two outputs point to the same input card
        if !used_before_indices.insert(trace.claimed_before_index) {
            return Err("Duplicate input index! Cheater attempted to clone a card.");
        }

        let point_after = &deck_after[trace.after_index];
        let point_before = &deck_before[trace.claimed_before_index];

        // 2. THE O(1) MILLER LOOP
        // e(card_after, -G2) * e(card_before, PK) == 1
        let is_match: bool = Bls12::multi_miller_loop(&[
            (point_after, &neg_g2_prepared),
            (point_before, &pk_prepared),
        ])
        .final_exponentiation()
        .is_identity()
        .into();

        if !is_match {
            return Err("Cryptographic forgery: The traced card does not mathematically match.");
        }
    }

    Ok(())
}
