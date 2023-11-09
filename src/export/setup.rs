use std::path::Path;

use crate::utilities::{r1cs::{ProverData, VerifierData},
    proof::serialize_into_file};

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
        Ok(())
    }

}