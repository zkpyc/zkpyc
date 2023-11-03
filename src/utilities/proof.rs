//! A trait for CirC-compatible proofs

use std::fs::File;
use std::path::Path;

use bincode::{deserialize_from, serialize_into};
use fxhash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};

use super::r1cs::{ProverData, VerifierData};
use circ::ir::term::text::parse_value_map;
use circ::ir::term::Value;


pub fn serialize_into_file<S: Serialize, P: AsRef<Path>>(data: &S, path: P) -> std::io::Result<()> {
    let mut file = File::create(path.as_ref())?;
    serialize_into(&mut file, data).unwrap();
    Ok(())
}

fn deserialize_from_file<D: for<'a> Deserialize<'a>, P: AsRef<Path>>(
    path: P,
) -> std::io::Result<D> {
    Ok(deserialize_from(File::open(path.as_ref())?).unwrap())
}

fn value_map_from_path<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<String, Value>> {
    Ok(parse_value_map(&std::fs::read(path)?))
}

/// A trait for CirC-compatible proofs
pub trait ProofSystem {
    /// A verifying key. Also used for commitments.
    type VerifyingKey: Serialize + for<'a> Deserialize<'a>;
    /// A proving key
    type ProvingKey: Serialize + for<'a> Deserialize<'a>;
    /// A proof
    type Proof: Serialize + for<'a> Deserialize<'a>;

    /// Setup
    fn setup(p_data: ProverData, v_data: VerifierData) -> (Self::ProvingKey, Self::VerifyingKey);
    /// Proving
    fn prove(pk: &Self::ProvingKey, witness: &HashMap<String, Value>) -> Self::Proof;
    /// Verification
    fn verify(vk: &Self::VerifyingKey, inst: &HashMap<String, Value>, pf: &Self::Proof) -> bool;

    /// Setup to files
    fn setup_fs(
        p_data: ProverData,
        v_data: VerifierData,
        pk_path: impl AsRef<Path>,
        vk_path: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        let (pk, vk) = Self::setup(p_data, v_data);
        serialize_into_file(&pk, pk_path)?;
        serialize_into_file(&vk, vk_path)?;
        Ok(())
    }
    /// Prove to/from files
    fn prove_fs(
        pk_path: impl AsRef<Path>,
        witness_path: impl AsRef<Path>,
        pf_path: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        let pk: Self::ProvingKey = deserialize_from_file(pk_path)?;
        let witness = value_map_from_path(witness_path)?;
        let pf = Self::prove(&pk, &witness);
        serialize_into_file(&pf, pf_path)
    }
    /// Verify from files
    fn verify_fs(
        vk_path: impl AsRef<Path>,
        instance_path: impl AsRef<Path>,
        pf_path: impl AsRef<Path>,
    ) -> std::io::Result<bool> {
        let instance = value_map_from_path(&instance_path)?;
        let vk: Self::VerifyingKey = deserialize_from_file(vk_path)?;
        let pf: Self::Proof = deserialize_from_file(pf_path)?;
        Ok(Self::verify(&vk, &instance, &pf))
    }
}

/// A commit-and-prove proof system.
pub trait CommitProofSystem {
    /// A verifying key. Also used for commitments.
    type VerifyingKey: Serialize + for<'a> Deserialize<'a>;
    /// A proving key
    type ProvingKey: Serialize + for<'a> Deserialize<'a>;
    /// A proof
    type Proof: Serialize + for<'a> Deserialize<'a>;
    /// A commitment to part of a witness.
    type Commitment: Serialize + for<'a> Deserialize<'a>;
    /// Randomness for a commitment.
    type ComRand: Serialize + for<'a> Deserialize<'a> + Default;
    /// Setup
    fn cp_setup(p_data: ProverData, v_data: VerifierData)
        -> (Self::ProvingKey, Self::VerifyingKey);
    /// Proving
    fn cp_prove(
        pk: &Self::ProvingKey,
        witness: &HashMap<String, Value>,
        rands: &[Self::ComRand],
    ) -> Self::Proof;
    /// Verification
    fn cp_verify(
        vk: &Self::VerifyingKey,
        inst: &HashMap<String, Value>,
        pf: &Self::Proof,
        cmts: &[Self::Commitment],
    ) -> bool;
    /// Commitment. The data should be a field-to-field array.
    fn cp_commit(vk: &Self::VerifyingKey, data: Value, rand: &Self::ComRand) -> Self::Commitment;
    /// Sample commitment randomness.
    fn sample_com_rand() -> Self::ComRand;

    /// Setup to files
    fn cp_setup_fs(
        p_data: ProverData,
        v_data: VerifierData,
        pk_path: impl AsRef<Path>,
        vk_path: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        let (pk, vk) = Self::cp_setup(p_data, v_data);
        serialize_into_file(&pk, pk_path)?;
        serialize_into_file(&vk, vk_path)?;
        Ok(())
    }
    /// Prove to/from files
    fn cp_prove_fs(
        pk_path: impl AsRef<Path>,
        witness_path: impl AsRef<Path>,
        pf_path: impl AsRef<Path>,
        rand_paths: Vec<impl AsRef<Path>>,
    ) -> std::io::Result<()> {
        let pk: Self::ProvingKey = deserialize_from_file(pk_path)?;
        let witness = value_map_from_path(witness_path)?;
        let mut rands: Vec<Self::ComRand> = Vec::new();
        for p in rand_paths {
            rands.push(deserialize_from_file(p)?);
        }
        let pf = Self::cp_prove(&pk, &witness, &rands);
        serialize_into_file(&pf, pf_path)
    }
    /// Verify from files
    fn cp_verify_fs(
        vk_path: impl AsRef<Path>,
        instance_path: impl AsRef<Path>,
        pf_path: impl AsRef<Path>,
        cmt_paths: Vec<impl AsRef<Path>>,
    ) -> std::io::Result<bool> {
        let instance = value_map_from_path(instance_path)?;
        let vk: Self::VerifyingKey = deserialize_from_file(vk_path)?;
        let pf: Self::Proof = deserialize_from_file(pf_path)?;
        let mut cmts: Vec<Self::Commitment> = Vec::new();
        for p in cmt_paths {
            cmts.push(deserialize_from_file(p)?);
        }
        Ok(Self::cp_verify(&vk, &instance, &pf, &cmts))
    }
    /// Commitment. The data should be a field-to-field array.
    fn cp_commit_fs(
        vk_path: impl AsRef<Path>,
        data_path: impl AsRef<Path>,
        rand_path: impl AsRef<Path>,
        cmt_path: impl AsRef<Path>,
    ) -> std::io::Result<()> {
        let vk: Self::VerifyingKey = deserialize_from_file(vk_path)?;
        let data_map = value_map_from_path(data_path)?;
        assert_eq!(1, data_map.len());
        let data = data_map.into_iter().next().unwrap().1;
        let rand: Self::ComRand = deserialize_from_file(rand_path)?;
        let cmt = Self::cp_commit(&vk, data, &rand);
        serialize_into_file(&cmt, cmt_path)
    }
    /// Sample commitment randomness.
    fn sample_com_rand_fs(rand_path: impl AsRef<Path>) -> std::io::Result<()> {
        let r = Self::sample_com_rand();
        serialize_into_file(&r, rand_path)
    }
}

impl<P: CommitProofSystem> ProofSystem for P {
    type VerifyingKey = <P as CommitProofSystem>::VerifyingKey;
    type ProvingKey = <P as CommitProofSystem>::ProvingKey;
    type Proof = <P as CommitProofSystem>::Proof;

    fn setup(p_data: ProverData, v_data: VerifierData) -> (Self::ProvingKey, Self::VerifyingKey) {
        assert_eq!(
            0,
            p_data.num_commitments(),
            "This predicate has commitments---use a CP proof system"
        );
        assert_eq!(
            0,
            v_data.num_commitments(),
            "This predicate has commitments---use a CP proof system"
        );
        Self::cp_setup(p_data, v_data)
    }

    fn prove(pk: &Self::ProvingKey, witness: &HashMap<String, Value>) -> Self::Proof {
        Self::cp_prove(pk, witness, &[])
    }

    fn verify(vk: &Self::VerifyingKey, inst: &HashMap<String, Value>, pf: &Self::Proof) -> bool {
        Self::cp_verify(vk, inst, pf, &[])
    }
}
