use circ::cfg::{
    clap::{self, Parser, ValueEnum},
    CircOpt,
};
use std::path::PathBuf;
use zkpyc::{export, utilities::scalar_fields::PrimeField};
use circ::cfg::cfg;

#[cfg(feature = "bellman")]
use bls12_381::Bls12;
#[cfg(feature = "bellman")]
use zkpyc::utilities::{bellman::Bellman, mirage::Mirage, proof::ProofSystem, r1cs::{ProverData, VerifierData}};

#[cfg(feature = "spartan")]
use circ::ir::term::text::parse_value_map;
#[cfg(feature = "spartan")]
use circ::target::r1cs::spartan;

use zkpyc::utilities::scalar_fields::bls12_381::Bls12_381;
use zkpyc::utilities::scalar_fields::bn256::Bn256;
use curve25519_dalek::scalar::Scalar as Curve25519;


#[derive(Debug, Parser)]
#[command(name = "zk", about = "The CirC ZKP runner")]
struct Options {
    #[arg(long, default_value = "P")]
    prover_key: PathBuf,
    #[arg(long, default_value = "V")]
    verifier_key: PathBuf,
    #[arg(long, default_value = "pi")]
    proof: PathBuf,
    #[arg(long, default_value = "in")]
    inputs: PathBuf,
    #[arg(long, default_value = "pin")]
    pin: PathBuf,
    #[arg(long, default_value = "vin")]
    vin: PathBuf,
    #[arg(long, default_value = "groth16")]
    proof_impl: ProofImpl,
    #[arg(long)]
    action: ProofAction,
    #[command(flatten)]
    circ: CircOpt,
}

#[derive(PartialEq, Debug, Clone, ValueEnum)]
/// `Prove`/`Verify` execute proving/verifying in bellman separately
/// `Spartan` executes both proving/verifying in spartan
enum ProofAction {
    Prove,
    Verify,
    Spartan,
}

#[derive(PartialEq, Debug, Clone, ValueEnum)]
/// Whether to use Groth16 or Mirage
enum ProofImpl {
    Groth16,
    Mirage,
    ZkInterface,
}

enum Modulus {
    Integer(rug::Integer)
}

fn prepare_prover_statements<F: PrimeField>(opts: &Options) {
    let pd: ProverData = bincode::deserialize_from(std::fs::File::open::<&std::path::Path>(&opts.prover_key.as_ref()).unwrap()).unwrap();
    let witness = circ::ir::term::text::parse_value_map(&std::fs::read(&opts.inputs).unwrap());
    export::write_constraints::<F>(&pd.r1cs, "function");
    let (
        public_inputs_arr,
        private_inputs_arr,
    ) = export::prepare_generate_proof::<F>(&pd.r1cs.vars, &pd.precompute, witness.clone());
    let first_local_id = public_inputs_arr.len() as u64;
    let free_variable_id = first_local_id + private_inputs_arr.len() as u64;
    export::write_circuit_header::<F>(first_local_id, free_variable_id, Some(&public_inputs_arr), "function");
    export::write_witnesses::<F>(first_local_id, &private_inputs_arr);
}

fn prepare_verifier_statements<F: PrimeField>(opts: &Options) {
    let vd: VerifierData = bincode::deserialize_from(std::fs::File::open::<&std::path::Path>(&opts.verifier_key.as_ref()).unwrap()).unwrap();
    let witness = circ::ir::term::text::parse_value_map(&std::fs::read(&opts.inputs).unwrap());
    export::write_constraints::<F>(&vd.r1cs, "function");
    let (
        public_inputs_arr,
        first_local_id,
        free_variable_id,
    ) = export::prepare_verify_proof::<F>(&vd.r1cs.vars, &vd.precompute, witness.clone());
    export::write_circuit_header::<F>(first_local_id, free_variable_id, Some(&public_inputs_arr), "function");
}


fn main() {
    let bls12_381_const = rug::Integer::from_str_radix("52435875175126190479447740508185965837690552500527637822603658699938581184513", 10).unwrap();
    let bn256_const = rug::Integer::from_str_radix("21888242871839275222246405745257275088548364400416034343698204186575808495617", 10).unwrap();
    let curve25519_const = rug::Integer::from_str_radix("7237005577332262213973186563042994240857116359379907606001950938285454250989", 10).unwrap();

    env_logger::Builder::from_default_env()
        .format_level(false)
        .format_timestamp(None)
        .init();
    let opts = Options::parse();
    circ::cfg::set(&opts.circ);
    match (&opts.action, &opts.proof_impl) {
        #[cfg(feature = "bellman")]
        (ProofAction::Prove, ProofImpl::Groth16) => {
            println!("Proving");
            Bellman::<Bls12>::prove_fs(opts.prover_key, opts.inputs, opts.proof).unwrap();
        }
        #[cfg(feature = "bellman")]
        (ProofAction::Prove, ProofImpl::Mirage) => {
            println!("Proving");
            Mirage::<Bls12>::prove_fs(opts.prover_key, opts.inputs, opts.proof).unwrap();
        }
        (ProofAction::Prove, ProofImpl::ZkInterface) => {
            println!("Generating Zkif Circuit, Constraints and Witnesses");
            // println!("{:#?}", cfg().field().modulus());
            match Modulus::Integer(cfg().field().modulus().clone()) {
                Modulus::Integer(i) if i == bls12_381_const => prepare_prover_statements::<Bls12_381>(&opts),
                Modulus::Integer(i) if i == bn256_const => prepare_prover_statements::<Bn256>(&opts),
                Modulus::Integer(i) if i == curve25519_const => prepare_prover_statements::<Curve25519>(&opts),
                _ => panic!("Prime field modulus not supported. The currently supported scalar fields are those of the  BLS12_381, BN256 and Curve25519 curves."),
            }
        }
        #[cfg(feature = "bellman")]
        (ProofAction::Verify, ProofImpl::Groth16) => {
            println!("Verifying");
            // // TEMP BLOCK
            // let vk: VerifyingKey<Bls12> = bincode::deserialize_from(std::fs::File::open::<&std::path::Path>(&opts.verifier_key.as_ref()).unwrap()).unwrap();

            // let witness = circ::ir::term::text::parse_value_map(&std::fs::read(&opts.inputs).unwrap());

            // // Normally the verifier also needs a copy of the constraints,
            // // but since all verifiers will also run the compiler, we will
            // // re-use the ProverKey.
            // export::write_constraints::<Bls12>(&vk.0.r1cs, witness.clone());

            // let (
            //     public_inputs_arr,
            //     first_local_id,
            //     free_variable_id,
            // ) = export::prepare_verify_proof(&vk, witness.clone());

            // export::write_circuit::<Bls12>(first_local_id, free_variable_id, Some(&public_inputs_arr), false);
            

            assert!(
                Bellman::<Bls12>::verify_fs(opts.verifier_key, opts.inputs, opts.proof).unwrap(),
                "invalid proof"
            );
        }
        #[cfg(feature = "bellman")]
        (ProofAction::Verify, ProofImpl::Mirage) => {
            println!("Verifying");
            assert!(
                Mirage::<Bls12>::verify_fs(opts.verifier_key, opts.inputs, opts.proof).unwrap(),
                "invalid proof"
            );
        }
        (ProofAction::Verify, ProofImpl::ZkInterface) => {
            println!("Generating Zkif Circuit and Constraints");
            match Modulus::Integer(cfg().field().modulus().clone()) {
                Modulus::Integer(i) if i == bls12_381_const => prepare_verifier_statements::<Bls12_381>(&opts),
                Modulus::Integer(i) if i == bn256_const => prepare_verifier_statements::<Bn256>(&opts),
                Modulus::Integer(i) if i == curve25519_const => prepare_verifier_statements::<Curve25519>(&opts),
                _ => panic!("Prime field modulus not supported. The currently supported scalar fields are those of the  BLS12_381, BN256 and Curve25519 curves."),
            }
        }
        #[cfg(not(feature = "bellman"))]
        (ProofAction::Prove | ProofAction::Verify, _) => panic!("Missing feature: bellman"),
        #[cfg(feature = "spartan")]
        (ProofAction::Spartan, _) => {
            let prover_input_map = parse_value_map(&std::fs::read(opts.pin).unwrap());
            println!("Spartan Proving");
            let (gens, inst, proof) = spartan::prove(opts.prover_key, &prover_input_map).unwrap();

            let verifier_input_map = parse_value_map(&std::fs::read(opts.vin).unwrap());
            println!("Spartan Verifying");
            spartan::verify(opts.verifier_key, &verifier_input_map, &gens, &inst, proof).unwrap();
        }
        #[cfg(not(feature = "spartan"))]
        (ProofAction::Spartan, _) => panic!("Missing feature: spartan"),
    }
}
