use std::{path::Path, error::Error};
use crate::utilities::{r1cs::{ProverData, VerifierData},
    proof::serialize_into_file};
use crate::utilities::scalar_fields::bls12_381::Bls12_381;
use crate::utilities::scalar_fields::bn256::Bn256;
use curve25519_dalek::scalar::Scalar as Curve25519;
use circ::cfg::cfg;

use super::write_constraints;

enum Modulus {
    Integer(rug::Integer)
}

pub struct ZkInterface;

impl ZkInterface {

    /// Setup to files
    pub fn setup_fs(
        p_data: ProverData,
        v_data: VerifierData,
        pk_path: impl AsRef<Path>,
        vk_path: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        serialize_into_file(&p_data, pk_path)?;
        serialize_into_file(&v_data, vk_path)?;

        let bls12_381_const = rug::Integer::from_str_radix("52435875175126190479447740508185965837690552500527637822603658699938581184513", 10).unwrap();
        let bn256_const = rug::Integer::from_str_radix("21888242871839275222246405745257275088548364400416034343698204186575808495617", 10).unwrap();
        let curve25519_const = rug::Integer::from_str_radix("7237005577332262213973186563042994240857116359379907606001950938285454250989", 10).unwrap();    

        let workspace = "zkif_export".as_ref();

        let result = match Modulus::Integer(cfg().field().modulus().clone()) {
            Modulus::Integer(i) if i == bls12_381_const => write_constraints::<Bls12_381>(&p_data.r1cs, "main", &workspace),
            Modulus::Integer(i) if i == bn256_const => write_constraints::<Bn256>(&p_data.r1cs, "main", &workspace),
            Modulus::Integer(i) if i == curve25519_const => write_constraints::<Curve25519>(&p_data.r1cs, "main", &workspace),
            _ => panic!("Prime field modulus not supported. The currently supported scalar fields are those of the  BLS12_381, BN256 and Curve25519 curves."),
        };

        Ok(())
    }

}