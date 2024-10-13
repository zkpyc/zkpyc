use bellman::{
    Circuit,
    ConstraintSystem,
    groth16::{
        generate_random_parameters,
        create_random_proof,
        prepare_verifying_key,
        verify_proof,
        Parameters,
        Proof,
    },
    SynthesisError,
    Variable,
    gadgets::num::AllocatedNum,
    gadgets::test::TestConstraintSystem,
};
use rand;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use super::import::{enforce, read_scalar};
pub use zkinterface::Reader;
use std::error::Error;
use ff::PrimeField;
use bls12_381::{Bls12, Scalar as Bls12Scalar};

const DEFAULT_KEY_PATH: &str = "bellman-pk";
const DEFAULT_PROOF_PATH: &str = "bellman-proof";


/// A circuit instance built from zkif messages.
#[derive(Clone, Debug)]
pub struct ZKIFCircuit<'a> {
    pub reader: &'a Reader,
}

impl<'a, Scalar: PrimeField> Circuit<Scalar> for ZKIFCircuit<'a> {
    fn synthesize<CS: ConstraintSystem<Scalar>>(self, cs: &mut CS) -> Result<(), SynthesisError>
    {
        // Check that we are working on the right field.
        match self.reader.first_header().unwrap().field_maximum() {
            None => {
                eprintln!("Warning: no field_maximum specified in messages, the field may be incompatible.");
            }
            Some(field_maximum) => {
                let requested: Scalar = read_scalar(field_maximum);
                let supported: Scalar = Scalar::one().neg();
                if requested != supported {
                    eprintln!("Error: This proving system does not support the field specified for this circuit.");
                    eprintln!("Requested field: {:?}", requested);
                    eprintln!("Supported field: {:?}", supported);
                    panic!();
                }
            }
        }

        // Track variables by id. Used to convert constraints.
        let mut id_to_var = HashMap::<u64, Variable>::new();

        id_to_var.insert(0, CS::one());

        // Allocate public inputs, with optional values.
        let public_vars = self.reader.instance_variables().unwrap();

        for var in public_vars {
            let mut cs = cs.namespace(|| format!("public_{}", var.id));
            let num = AllocatedNum::alloc(&mut cs, || {
                Ok(read_scalar(var.value))
            })?;

            num.inputize(&mut cs)?;

            // Track input variable.
            id_to_var.insert(var.id, num.get_variable());
        }

        // Allocate private variables, with optional values.
        let private_vars = self.reader.private_variables().unwrap();

        for var in private_vars {
            let num = AllocatedNum::alloc(
                cs.namespace(|| format!("private_{}", var.id)), || {
                    Ok(read_scalar(var.value))
                })?;

            // Track private variable.
            id_to_var.insert(var.id, num.get_variable());
        };

        for (i, constraint) in self.reader.iter_constraints().enumerate() {
            enforce(&mut cs.namespace(|| format!("constraint_{}", i)), &id_to_var, &constraint);
        }

        Ok(())
    }
}


pub fn validate<Scalar: PrimeField>(
    reader: &Reader,
    print: bool,
) -> Result<(), Box<dyn Error>> {
    let circuit = ZKIFCircuit { reader };
    let mut cs = TestConstraintSystem::<Scalar>::new();
    circuit.synthesize(&mut cs)?;

    if print {
        eprintln!("{}", cs.pretty_print());
    }

    match cs.which_is_unsatisfied() {
        None => {
            eprintln!("Satisfied: YES");
            Ok(())
        }
        Some(constraint) => {
            eprintln!("Satisfied: NO");
            eprintln!("This constraint is not satisfied: {}", constraint);
            Err("The witness does not satisfy the constraints.".into())
        }
    }
}


pub fn setup(
    reader: &Reader,
    workspace: &Path,
    key_name: &str,
) -> Result<(), Box<dyn Error>>
{
    let key_path = workspace.join(key_name);

    let circuit = ZKIFCircuit { reader };

    let mut rng = rand::thread_rng();
    let params = generate_random_parameters::<Bls12, _, _>(
        circuit.clone(),
        &mut rng,
    )?;

    // Store params.
    let file = File::create(&key_path)?;
    params.write(file)?;
    // eprintln!("Written parameters into {}", key_path.display());

    Ok(())
}

pub fn prove(
    reader: &Reader,
    workspace: &Path,
    key_name: &str,
    proof_name: &str,
) -> Result<(), Box<dyn Error>>
{
    let key_path = workspace.join(key_name);
    let proof_path = workspace.join(proof_name);

    let circuit = ZKIFCircuit { reader };

    // Load params.
    let params = {
        // eprintln!("Reading parameters from {}", key_path.display());
        let mut file = File::open(&key_path)?;
        Parameters::<Bls12>::read(&mut file, false)?
    };

    let mut rng = rand::thread_rng();
    let proof = create_random_proof(
        circuit,
        &params,
        &mut rng,
    )?;

    // Store proof.
    let file = File::create(&proof_path)?;
    proof.write(file)?;
    // eprintln!("Written proof into {}", proof_path.display());

    Ok(())
}

pub fn verify(
    reader: &Reader,
    workspace: &Path,
    key_name: &str,
    proof_name: &str,
) -> Result<bool, Box<dyn Error>> {
    let key_path = workspace.join(key_name);
    let proof_path = workspace.join(proof_name);

    let pvk = {
        // eprintln!("Reading parameters from {}", key_path.display());
        let mut file = File::open(&key_path)?;
        let params = Parameters::<Bls12>::read(&mut file, false)?;
        prepare_verifying_key::<Bls12>(&params.vk)
    };

    let public_inputs: Vec<Bls12Scalar> = {
        match reader.instance_variables() {
            None => Vec::new(),
            Some(instance_variables) => {
                instance_variables.iter().map(|var|
                    read_scalar(var.value)
                ).collect()
            }
        }
    };

    let proof = {
        // eprintln!("Reading proof from {}", proof_path.display());
        let mut file = File::open(&proof_path)?;
        Proof::read(&mut file).unwrap()
    };
    let res = verify_proof(&pvk, &proof, &public_inputs);

    match res {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}


#[test]
fn test_zkif_backend() -> Result<(), Box<dyn Error>> {

    // Load test messages.
    let test_dir = Path::new("src/tests/example.zkif");
    let out_dir = Path::new("local");

    let mut reader = Reader::new();
    reader.read_file(test_dir)?;

    validate::<bls12_381::Scalar>(&reader, false)?;

    setup(&reader, out_dir, DEFAULT_KEY_PATH)?;

    prove(&reader, out_dir, DEFAULT_KEY_PATH, DEFAULT_PROOF_PATH)?;

    verify(&reader, out_dir, DEFAULT_KEY_PATH, DEFAULT_PROOF_PATH)?;

    Ok(())
}
