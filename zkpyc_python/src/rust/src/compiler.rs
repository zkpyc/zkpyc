use pyo3::{prelude::*, exceptions};

use circ::ir::{opt::Opt, opt::opt, term::Computations};
use circ_opt::CircOpt;
use circ::cfg::cfg;
use zkpyc::export::{write_constraints, prepare_prover_statements, prepare_verifier_statements};
use zkpyc::front::{self, Mode::Proof, FrontEnd, python::Inputs};
use zkpyc::utilities::r1cs::{ProverData, VerifierData};
use zkpyc::utilities::proof::serialize_into_file;
use zkpyc::utilities::scalar_fields::PrimeField;
use zkpyc::utilities::{opt::reduce_linearities, trans::to_r1cs};
use zkpyc::utilities::scalar_fields::bls12_381::Bls12_381;
use zkpyc::utilities::scalar_fields::bn256::Bn256;
use curve25519_dalek::scalar::Scalar as Curve25519;
use std::fs::{File, remove_file, self};
use std::io::Write;
use std::panic;
use std::path::{Path, PathBuf};

use crate::ff_constants::*;
use crate::utilities::{create_folder, rename_zkif_file};

enum Modulus {
    Integer(rug::Integer)
}

trait ProverOrVerifier {
    fn identifier() -> &'static str;
    fn input_type() -> &'static str;
    fn prepare_statements<F: PrimeField>(
        f_name: &str,
        module_name: &str,
        inputs_path: &Path,
        pd_or_vd_path: &Path,
        zkif_workspace: &Path,
        is_prover: bool,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

struct Prover;

struct Verifier;

impl ProverOrVerifier for Prover {
    fn identifier() -> &'static str {
        "prover"
    }

    fn input_type() -> &'static str {
        "pin"
    }

    fn prepare_statements<F: PrimeField>(
        f_name: &str,
        module_name: &str,
        inputs_path: &Path,
        key_path: &Path,
        zkif_workspace: &Path,
        generate_constraints: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        prepare_prover_statements::<F>(&f_name, &inputs_path, &key_path, &zkif_workspace, generate_constraints)?;
        let new_witness_name = format!("witness_{}_{}", module_name, f_name);
        let new_header_name = format!("header_{}_{}", module_name, f_name);
        let new_constraints_name = format!("constraints_{}_{}", module_name, f_name);
        rename_zkif_file("witness", &new_witness_name, &zkif_workspace)?;
        rename_zkif_file("header", &new_header_name, &zkif_workspace)?;
        if generate_constraints {
            rename_zkif_file("constraints_0", &new_constraints_name, &zkif_workspace)?;
        }
        Ok(())
    }
}

impl ProverOrVerifier for Verifier {
    fn identifier() -> &'static str {
        "verifier"
    }

    fn input_type() -> &'static str {
        "vin"
    }

    fn prepare_statements<F: PrimeField>(
        f_name: &str,
        module_name: &str,
        inputs_path: &Path,
        key_path: &Path,
        zkif_workspace: &Path,
        generate_constraints: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        prepare_verifier_statements::<F>(&f_name, &inputs_path, &key_path, &zkif_workspace, generate_constraints)?;
        let new_header_name = format!("header_{}_{}", module_name, f_name);
        let new_constraints_name = format!("constraints_{}_{}", module_name, f_name);
        rename_zkif_file("header", &new_header_name, &zkif_workspace)?;
        if generate_constraints {
            rename_zkif_file("constraints_0", &new_constraints_name, &zkif_workspace)?;
        }
        Ok(())
    }
}

fn optimize_computations(cs: Computations) -> Computations {
    let mut opts = Vec::new();

    opts.push(Opt::ScalarizeVars);
    opts.push(Opt::Flatten);
    opts.push(Opt::Sha);
    opts.push(Opt::ConstantFold(Box::new([])));
    opts.push(Opt::ParseCondStores);
    // Tuples must be eliminated before oblivious array elim
    opts.push(Opt::Tuple);
    opts.push(Opt::ConstantFold(Box::new([])));
    opts.push(Opt::Tuple);
    opts.push(Opt::Obliv);
    // The obliv elim pass produces more tuples, that must be eliminated
    opts.push(Opt::Tuple);
    // The following optimizations are run when the ram option is enabled
    // We will assume it is true by default because cfg() does not expose
    // CircOpt yet (it is still private), and a workaround isn't worth it.
    opts.push(Opt::PersistentRam);
    opts.push(Opt::VolatileRam);
    opts.push(Opt::SkolemizeChallenges);
    opts.push(Opt::LinearScan);
    // The linear scan pass produces more tuples, that must be eliminated
    opts.push(Opt::Tuple);
    opts.push(Opt::Flatten);
    opts.push(Opt::ConstantFold(Box::new([])));
    opt(cs, opts)
}

fn run_zkpyc_compiler(
    f_name: &String,
    inputs: Inputs,
) -> PyResult<(ProverData, VerifierData, usize)> {    
    let cs = front::python::PythonFE::gen(inputs);
    let cs = optimize_computations(cs);
    let cs = cs.get(f_name);
    let mut r1cs = to_r1cs(cs, cfg());
    r1cs = reduce_linearities(r1cs, cfg());
    let constraints_count = r1cs.constraints().len();
    let (prover_data, verifier_data) = r1cs.finalize(cs);
    Ok((prover_data, verifier_data, constraints_count))
}

#[pyfunction]
#[pyo3(signature = (
    modulus="52435875175126190479447740508185965837690552500527637822603658699938581184513",
))]
fn init(modulus: &str) -> PyResult<()> {
    let mut circ_options = CircOpt::default();
    circ_options.field.custom_modulus = String::from(modulus);
    // We will make the circ_options customizable in the future.
    circ::cfg::set(&circ_options);
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (id=0))]
fn cleanup(id: usize) -> PyResult<()> {
    let workspace = create_folder(Path::new("."), &format!("cache_id_{}", id));
    Ok(fs::remove_dir_all(workspace)?)
}

#[pyfunction]
#[pyo3(signature = (f_name, input, id=0, module_name=String::from("__main__")))]
fn compile(
    _py: Python,
    f_name: String,
    input: String,
    id: usize,
    module_name: String,
) -> PyResult<usize> {
    // Define directory where ZKP data will be stored
    let workspace = create_folder(Path::new("."), &format!("cache_id_{}", id));
    
    let file_path = Path::new(".").join(PathBuf::from(format!(".id_{}_{}_{}.py", id, module_name, f_name)));
    let mut file = File::create(&file_path)?;
    // Because of how the compiler is written, we need to temporarily
    // store source code as a file.
    file.write_all(input.as_bytes())?;

    let inputs = front::python::Inputs {
        file: file_path.clone(),
        entry_point: f_name.clone(),
        mode: Proof,
    };

    // Run ZKPyC and catch panic or other PyErrors
    panic::set_hook(Box::new(|_info| {
        // do nothing
    }));
    let result = panic::catch_unwind(|| run_zkpyc_compiler(&f_name, inputs));

    // Remove temporary function definition file.
    let (pd, vd, constr_count) = match result {
        Ok(Ok(res)) => {
            remove_file(&file_path)?;
            res
        }
        Ok(Err(err)) => {
            remove_file(&file_path)?;
            return Err(err);
        }
        Err(panic_payload) => {
            // Try to extract the panic message from the payload
            let panic_msg = if let Some(msg) = panic_payload.downcast_ref::<&str>() {
                msg.to_string()
            } else if let Some(msg) = panic_payload.downcast_ref::<String>() {
                msg.clone()
            } else {
                "Unknown panic message".to_string()
            };

            // Return the panic message as a Python error
            // return Err(PyErr::new::<exc::RuntimeError, _>(_py, panic_msg));
            return Err(exceptions::PySyntaxError::new_err(panic_msg));
        }
    };

    let zkif_workspace = create_folder(&workspace, "zkif_export");
    match Modulus::Integer(cfg().field().modulus().clone()) {
        Modulus::Integer(i) if i == get_bls12_381_const() => write_constraints::<Bls12_381>(&pd.r1cs, &f_name, &zkif_workspace),
        Modulus::Integer(i) if i == get_bn256_const() => write_constraints::<Bn256>(&pd.r1cs, &f_name, &zkif_workspace),
        Modulus::Integer(i) if i == get_curve25519_const() => write_constraints::<Curve25519>(&pd.r1cs, &f_name, &zkif_workspace),
        _ => panic!("Prime field modulus not supported. The currently supported scalar fields are those of the BLS12_381, BN256 and Curve25519 curves."),
    }.map_err(|err| {
        exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))
    })?;

    // Change the zkif name to contain information about the module and function name.
    let new_constraints_name = format!("constraints_{}_{}", module_name, f_name);
    let new_header_name = format!("header_{}_{}", module_name, f_name);
    rename_zkif_file("constraints_0", &new_constraints_name, &zkif_workspace)?;
    rename_zkif_file("header", &new_header_name, &zkif_workspace)?;


    let zkp_data_workspace = create_folder(&workspace, "zkp_data");
    let pd_path = zkp_data_workspace.join(format!("{}_{}_prover_data.dat", module_name, f_name));
    let vd_path = zkp_data_workspace.join(format!("{}_{}_verifier_data.dat", module_name, f_name));

    serialize_into_file(&pd, pd_path)?;
    serialize_into_file(&vd, vd_path)?;
    
    Ok(constr_count)
}

fn setup_proof_or_verification<PV: ProverOrVerifier>(
    _py: Python,
    f_name: String,
    input: String,
    id: usize,
    module_name: String,
) -> PyResult<()> {
    let workspace = create_folder(Path::new("."), &format!("cache_id_{}", id));
    let zkif_workspace = create_folder(&workspace, "zkif_export");
    let zkp_data_workspace = create_folder(&workspace, "zkp_data");

    let identifier = PV::identifier();
    let pd_or_vd_path = zkp_data_workspace.join(format!("{}_{}_{}_data.dat", module_name, f_name, identifier));
    let inputs_path = Path::new(".").join(PathBuf::from(format!(".id_{}_{}_{}.py.{}", id, module_name, f_name, PV::input_type())));
    let mut file = File::create(&inputs_path)?;
    file.write_all(input.as_bytes())?;

    // Run verifier or proof statement setup and catch panic or other PyErrors
    panic::set_hook(Box::new(|_info| {}));

    let result = panic::catch_unwind(|| {
        match Modulus::Integer(cfg().field().modulus().clone()) {
            Modulus::Integer(i) if i == get_bls12_381_const() => PV::prepare_statements::<Bls12_381>(&f_name, &module_name, &inputs_path, &pd_or_vd_path, &zkif_workspace, false),
            Modulus::Integer(i) if i == get_bn256_const() => PV::prepare_statements::<Bn256>(&f_name, &module_name, &inputs_path, &pd_or_vd_path, &zkif_workspace, false),
            Modulus::Integer(i) if i == get_curve25519_const() => PV::prepare_statements::<Curve25519>(&f_name, &module_name, &inputs_path, &pd_or_vd_path, &zkif_workspace, false),
            _ => panic!("Prime field modulus not supported. The currently supported scalar fields are those of the BLS12_381, BN256 and Curve25519 curves."),
        }.map_err(|err| {
            exceptions::PyRuntimeError::new_err(format!("An error occurred: {}", err))
        })
    });

    match result {
        Ok(Ok(_)) => {
            remove_file(&inputs_path)?;
            Ok(())
        }
        Ok(Err(err)) => {
            remove_file(&inputs_path)?;
            Err(err)
        }
        Err(panic_payload) => {
            let panic_msg = if let Some(msg) = panic_payload.downcast_ref::<&str>() {
                msg.to_string()
            } else if let Some(msg) = panic_payload.downcast_ref::<String>() {
                msg.clone()
            } else {
                "Unknown panic message".to_string()
            };
            Err(exceptions::PySyntaxError::new_err(panic_msg))
        }
    }
}

#[pyfunction]
#[pyo3(signature = (f_name, input, id=0, module_name=String::from("__main__")))]
fn setup_proof(
    _py: Python,
    f_name: String,
    input: String,
    id: usize,
    module_name: String,
) -> PyResult<()> {
    setup_proof_or_verification::<Prover>(_py, f_name, input, id, module_name)
}

#[pyfunction]
#[pyo3(signature = (f_name, input, id=0, module_name=String::from("__main__")))]
fn setup_verification(
    _py: Python,
    f_name: String,
    input: String,
    id: usize,
    module_name: String,
) -> PyResult<()> {
    setup_proof_or_verification::<Verifier>(_py, f_name, input, id, module_name)
}

pub(crate) fn create_submodule(py: pyo3::Python<'_>) -> pyo3::PyResult<&pyo3::prelude::PyModule> {
    let submod = pyo3::prelude::PyModule::new(py, "compiler")?;
    submod.add_function(pyo3::wrap_pyfunction!(init, submod)?)?;
    submod.add_function(pyo3::wrap_pyfunction!(compile, submod)?)?;
    submod.add_function(pyo3::wrap_pyfunction!(cleanup, submod)?)?;
    submod.add_function(pyo3::wrap_pyfunction!(setup_proof, submod)?)?;
    submod.add_function(pyo3::wrap_pyfunction!(setup_verification, submod)?)?;
    Ok(submod)
}
