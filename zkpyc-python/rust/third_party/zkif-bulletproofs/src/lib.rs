

extern crate byteorder;
extern crate core;
extern crate digest;
extern crate rand;
extern crate sha3;

extern crate clear_on_drop;
extern crate curve25519_dalek;
extern crate merlin;
extern crate subtle;
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate failure;

#[cfg(test)]
extern crate bincode;

mod util;

mod notes {
    mod inner_product_proof {}
    mod range_proof {}
    mod r1cs_proof {}
}

mod errors;
mod generators;
mod inner_product_proof;
mod range_proof;
mod transcript;

pub use errors::ProofError;
pub use generators::{BulletproofGens, BulletproofGensShare, PedersenGens};
pub use range_proof::RangeProof;

pub mod range_proof_mpc {
    pub use errors::MPCError;
    pub use range_proof::dealer;
    pub use range_proof::messages;
    pub use range_proof::party;
}

#[cfg(feature = "yoloproofs")]
pub mod r1cs;
