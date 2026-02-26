/// Shared Data Types
use bls12_381::{G1Affine, G2Affine, Scalar};

pub type SigningKey = Scalar;
pub type Signature = G1Affine;
pub type PublicKey = G2Affine;

