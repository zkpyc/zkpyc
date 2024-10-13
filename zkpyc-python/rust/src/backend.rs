use std::{path::Path, fs::File, io::{Write, Read}};
use bincode;
use pyo3::{prelude::*, exceptions, types::PyBytes};
use zkinterface::{Reader, consumers::stats::Stats, Workspace};
use zkinterface_bulletproofs::r1cs::R1CSProof;

use crate::utilities::create_folder;


#[pyfunction]
#[pyo3(signature = (circuit, constraints, f_name, id=0, module_name=String::from("__main___"), backend=None))]
fn setup(
    _py: Python,
    circuit: &PyBytes,
    constraints: &PyBytes,
    f_name: String,
    id: usize,
    module_name: String,
    backend: Option<String>,
) -> PyResult<()> {
    let mut reader = Reader::new();

    match reader.push_message(circuit.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    match reader.push_message(constraints.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    let workspace = Path::new(".").join(format!("cache_id_{}", id));
    let zkp_key_workspace = create_folder(&workspace, "zkp_params_and_proofs");
    let key_name = format!("{}_{}_key.dat", module_name, f_name);

    match backend {
        Some(s) => match s.as_str() {
            "groth16" => match zkinterface_bellman::zkif_backend::setup(&reader, &zkp_key_workspace, &key_name) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
            }
            e => Err(exceptions::PyValueError::new_err(format!("The backend: {}, is currently not supported.", e)))
        }
        None => Err(exceptions::PyValueError::new_err(format!("No backend provided for trusted setup.")))
    }
}

#[pyfunction]
#[pyo3(signature = (circuit, witness, constraints, f_name, id=0, module_name=String::from("__main___"), backend=None))]
fn prove(
    _py: Python,
    circuit: &PyBytes,
    witness: &PyBytes,
    constraints: &PyBytes,
    f_name: String,
    id: usize,
    module_name: String,
    backend: Option<String>,
) -> PyResult<()> {
    let mut reader = Reader::new();

    match reader.push_message(circuit.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    match reader.push_message(witness.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    match reader.push_message(constraints.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    let workspace = Path::new(".").join(format!("cache_id_{}", id));
    let zkp_key_workspace = create_folder(&workspace, "zkp_params_and_proofs");
    let zkif_workspace = workspace.join(format!("zkif_export"));
    let key_name = format!("{}_{}_key.dat", module_name, f_name);
    let proof_name = format!("{}_{}_proof.dat", module_name, f_name);

    // Unfortunately zkinterface consumers only read from files or stdin, so we define the cached zkif file paths.
    let circuit_file = zkif_workspace.join(format!("header_{}_{}.zkif", module_name, f_name));
    let witness_file = zkif_workspace.join(format!("witness_{}_{}.zkif", module_name, f_name));
    let constraints_file = zkif_workspace.join(format!("constraints_{}_{}.zkif", module_name, f_name));

    let mut stats = Stats::default();
    let ws = match Workspace::from_dirs_and_files(&[circuit_file, witness_file, constraints_file]) {
        Ok(ws) => ws,
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };
    stats.ingest_workspace(&ws);

    match backend {
        Some(s) => match s.as_str() {
            "groth16" => match zkinterface_bellman::zkif_backend::prove(&reader, &zkp_key_workspace, &key_name, &proof_name) {
                Ok(_) => Ok(()),
                Err(err) => Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
            }
            "bulletproofs" => {
                let generators_count = (stats.multiplications.next_power_of_two()*2) as usize;
                let proof_path = zkp_key_workspace.join(proof_name);
                let proof = match zkinterface_bulletproofs::r1cs::zkinterface_backend::prove(&reader, generators_count) {
                    Ok(pf) => pf,
                    Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
                };
                let proof_ser = match bincode::serialize(&proof) {
                    Ok(pf) => pf,
                    Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
                };
                File::create(proof_path)?.write_all(&proof_ser)?;
                Ok(())
            }
            e => Err(exceptions::PyValueError::new_err(format!("The backend: {}, is currently not supported.", e)))
        }
        None => Err(exceptions::PyValueError::new_err(format!("No backend provided for proof.")))
    }
}

#[pyfunction]
#[pyo3(signature = (circuit, constraints, f_name, id=0, module_name=String::from("__main___"), backend=None))]
fn verify(
    _py: Python,
    circuit: &PyBytes,
    constraints: &PyBytes,
    f_name: String,
    id: usize,
    module_name: String,
    backend: Option<String>,
) -> PyResult<bool> {
    let mut reader = Reader::new();

    match reader.push_message(circuit.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    match reader.push_message(constraints.as_bytes().to_vec()) {
        Ok(_) => (),
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };

    let workspace = Path::new(".").join(format!("cache_id_{}", id));
    let zkp_key_workspace = create_folder(&workspace, "zkp_params_and_proofs");
    let zkif_workspace = workspace.join(format!("zkif_export"));
    let key_name = format!("{}_{}_key.dat", module_name, f_name);
    let proof_name = format!("{}_{}_proof.dat", module_name, f_name);

    // Unfortunately zkinterface consumers only read from files or stdin, so we define the cached zkif file paths.
    let circuit_file = zkif_workspace.join(format!("header_{}_{}.zkif", module_name, f_name));
    let constraints_file = zkif_workspace.join(format!("constraints_{}_{}.zkif", module_name, f_name));

    let mut stats = Stats::default();
    let ws = match Workspace::from_dirs_and_files(&[circuit_file, constraints_file]) {
        Ok(ws) => ws,
        Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
    };
    stats.ingest_workspace(&ws);

    match backend {
        Some(s) => match s.as_str() {
            "groth16" => match zkinterface_bellman::zkif_backend::verify(&reader, &zkp_key_workspace, &key_name, &proof_name) {
                Ok(res) => Ok(res),
                Err(err) => Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
            }
            "bulletproofs" => {
                let generators_count = (stats.multiplications.next_power_of_two()*2) as usize;
                let proof_path = zkp_key_workspace.join(proof_name);
                
                // Load from file.
                let mut proof_ser = Vec::new();
                File::open(&proof_path)?.read_to_end(&mut proof_ser)?;
                let proof: R1CSProof = match bincode::deserialize(&proof_ser) {
                    Ok(pf) => pf,
                    Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
                };
                
                match zkinterface_bulletproofs::r1cs::zkinterface_backend::verify(&reader, &proof, generators_count) {
                    Ok(res) => Ok(res),
                    Err(err) => return Err(exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))),
                }
            }
            e => Err(exceptions::PyValueError::new_err(format!("The backend: {}, is currently not supported.", e)))
        }
        None => Err(exceptions::PyValueError::new_err(format!("No backend provided for proof.")))
    }
}

pub(crate) fn create_submodule(py: pyo3::Python<'_>) -> pyo3::PyResult<&pyo3::prelude::PyModule> {
    let submod = pyo3::prelude::PyModule::new(py, "backend")?;
    submod.add_function(pyo3::wrap_pyfunction!(setup, submod)?)?;
    submod.add_function(pyo3::wrap_pyfunction!(prove, submod)?)?;
    submod.add_function(pyo3::wrap_pyfunction!(verify, submod)?)?;
    Ok(submod)
}
