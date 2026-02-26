/// Signing & Secret (Un)Masking
use bls12_381::G1Affine;
use pairing::group::Curve;

use crate::{
    hash_to_curve::hash_to_curve,
    types::{Signature, SigningKey},
};

pub fn sign(data: &[u8], k: SigningKey) -> Signature {
    let mut p = hash_to_curve(data);
    p *= k;
    p.to_affine()
}

pub fn mask(g1: G1Affine, k: SigningKey) -> G1Affine {
    let p = g1 * k;
    p.to_affine()
}

pub fn unmask(g1: G1Affine, k: SigningKey) -> G1Affine {
    let i = k.invert().expect("Failed to invert");
    let u = g1 * i;
    u.to_affine()
}
