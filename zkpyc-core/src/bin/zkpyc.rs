#![allow(unused_imports)]
use circ_opt::{CircOpt,
    clap::Parser,
    clap::Subcommand,
    clap::ValueEnum,
    clap::Args,
};
use circ_opt::clap;
use zkpyc_core::export::setup::ZkInterface;
use std::io::Read;
use std::{env, io, path};
use std::path::{Path, PathBuf};
use zkpyc_core::front::{self, SourceInput};
use crate::front::{FrontEnd, Mode};
#[cfg(feature = "r1cs")]
use zkpyc_core::utilities::{trans::to_r1cs, opt::reduce_linearities};
use circ::ir::term::{Node, Op, BV_LSHR, BV_SHL};
use circ::ir::{
    opt::{opt, Opt},
    term::{
        check,
        text::{parse_value_map, serialize_value_map},
    },
};
#[cfg(feature = "bellman")]
use bellman::{
    gadgets::test::TestConstraintSystem,
    groth16::{
        create_random_proof, generate_parameters, generate_random_parameters,
        prepare_verifying_key, verify_proof, Parameters, Proof, VerifyingKey,
    },
    Circuit,
};
#[cfg(feature = "bellman")]
use bls12_381::{Bls12, Scalar};
// use curve25519_dalek::scalar::Scalar;
#[cfg(feature = "bellman")]
use zkpyc_core::utilities::{
    bellman::Bellman,
    mirage::Mirage,
    proof::{CommitProofSystem, ProofSystem},
};
use circ::cfg::cfg;
use log::trace;


#[derive(Parser, Debug)]
struct Options {
    #[command(flatten)]
    pub circ: CircOpt,

    #[arg(name = "PATH")]
    path: Option<PathBuf>, // If None, fall back to stdin

    #[command(flatten)]
    frontend: FrontendOptions,

    #[structopt(subcommand)]
    backend: Backend,
}

#[derive(PartialEq, Eq, Debug, Clone, ValueEnum)]
enum ProofAction {
    Count,
    Setup,
    CpSetup,
    SpartanSetup,
}

#[derive(PartialEq, Eq, Debug, Clone, ValueEnum)]
enum ProofImpl {
    Groth16,
    Mirage,
    ZkInterface,
}

#[derive(Debug, Args)]
struct FrontendOptions {
    /// Value threshold
    #[arg(long)]
    value_threshold: Option<u64>,
}

#[derive(Debug, Subcommand)]
enum Backend {
    R1cs {
        #[arg(long, default_value = "P")]
        prover_key: PathBuf,
        #[arg(long, default_value = "V")]
        verifier_key: PathBuf,
        #[arg(long, default_value = "50")]
        /// linear combination constraints up to this size will be eliminated
        lc_elimination_thresh: usize,
        #[arg(long, default_value = "count")]
        action: ProofAction,
        #[arg(long, default_value = "groth16")]
        proof_impl: ProofImpl,
    },
}

fn main() {
    env_logger::Builder::from_default_env()
        .format_level(false)
        .format_timestamp(None)
        .init();
    let options = Options::parse();
    circ::cfg::set(&options.circ);
    // let path_buf = options.path.unwrap();
    let mode = match options.backend {
        Backend::R1cs { .. } => match options.frontend.value_threshold {
            Some(t) => Mode::ProofOfHighValue(t),
            None => Mode::Proof,
        }
    };

    let source = options.path
    .as_ref()
    .map(|path| SourceInput::Path(path.clone()))
    .unwrap_or_else(|| {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle.read_to_string(&mut buffer).expect("Failed to read from stdin");

        let working_dir = env::current_dir().expect("Failed to get current working directory");

        SourceInput::String(buffer, PathBuf::default(), "<stdin>".to_owned())
    });

    let inputs = front::python::Inputs {
        source,
        entry_point: String::from("main"),
        mode,
    };

    let cs = front::python::PythonFE::gen(inputs);

    // TEMPORARY DEBUG
    // println!("{:#?}", cs);

    // now we run the compiler
    #[cfg(feature = "r1cs")]
    let cs = match mode {
        Mode::Opt => opt(
            cs,
            vec![Opt::ScalarizeVars, Opt::ConstantFold(Box::new([]))],
        ),
        Mode::Mpc(_) => {
            let ignore = [BV_LSHR, BV_SHL];
            opt(
                cs,
                vec![
                    Opt::ScalarizeVars,
                    Opt::Flatten,
                    Opt::Sha,
                    Opt::ConstantFold(Box::new(ignore.clone())),
                    Opt::Flatten,
                    // Function calls return tuples
                    Opt::Tuple,
                    Opt::Obliv,
                    // The obliv elim pass produces more tuples, that must be eliminated
                    Opt::Tuple,
                    Opt::LinearScan,
                    // The linear scan pass produces more tuples, that must be eliminated
                    Opt::Tuple,
                    Opt::ConstantFold(Box::new(ignore)),
                    // Binarize nary terms
                    Opt::Binarize,
                ],
                // vec![Opt::Sha, Opt::ConstantFold, Opt::Mem, Opt::ConstantFold],
            )
        }
        Mode::Proof | Mode::ProofOfHighValue(_) => {
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
            if options.circ.ram.enabled {
                opts.push(Opt::PersistentRam);
                opts.push(Opt::VolatileRam);
                opts.push(Opt::SkolemizeChallenges);
            }
            opts.push(Opt::LinearScan);
            // The linear scan pass produces more tuples, that must be eliminated
            opts.push(Opt::Tuple);
            opts.push(Opt::Flatten);
            opts.push(Opt::ConstantFold(Box::new([])));
            opt(cs, opts)
        }
    };
    println!("Done with IR optimization");

    #[cfg(feature = "r1cs")]
    match options.backend {
        Backend::R1cs {
            action,
            prover_key,
            verifier_key,
            proof_impl,
            ..
        } => {
            println!("Converting to r1cs");
            let cs = cs.get("main");
            trace!("IR: {}", circ::ir::term::text::serialize_computation(cs));
            let mut r1cs = to_r1cs(cs, cfg());

            // println!("R1CS");
            // println!("{:#?}", &r1cs);

            println!("Pre-opt R1cs size: {}", r1cs.constraints().len());
            r1cs = reduce_linearities(r1cs, cfg());

            // TEMPORARY DEBUG
            // println!("{:#?}", &r1cs);

            println!("Final R1cs size: {}", r1cs.constraints().len());
            let (prover_data, verifier_data) = r1cs.finalize(cs);
            
            // println!("R1CS");
            // println!("{:#?}", &prover_data);

            // println!("Verifier Data");
            // println!("{:#?}", &verifier_data);

            match action {
                ProofAction::Count => (),
                #[cfg(feature = "bellman")]
                ProofAction::Setup => {
                    println!("Generating Parameters");
                    match proof_impl {
                        ProofImpl::Groth16 => Bellman::<Bls12>::setup_fs(
                            prover_data,
                            verifier_data,
                            prover_key,
                            verifier_key,
                        )
                        .unwrap(),
                        ProofImpl::Mirage => Mirage::<Bls12>::setup_fs(
                            prover_data,
                            verifier_data,
                            prover_key,
                            verifier_key,
                        )
                        .unwrap(),
                        ProofImpl::ZkInterface => ZkInterface::setup_fs(
                            prover_data,
                            verifier_data,
                            prover_key,
                            verifier_key,
                        )
                        .unwrap(),
                    };
                }
                #[cfg(not(feature = "bellman"))]
                ProofAction::Setup => panic!("Missing feature: bellman"),
                #[cfg(feature = "bellman")]
                ProofAction::CpSetup => {
                    println!("Generating Parameters");
                    match proof_impl {
                        ProofImpl::Groth16 => panic!("Groth16 is not CP"),
                        ProofImpl::Mirage => Mirage::<Bls12>::cp_setup_fs(
                            prover_data,
                            verifier_data,
                            prover_key,
                            verifier_key,
                        )
                        .unwrap(),
                        ProofImpl::ZkInterface => todo!(),
                    };
                }
                #[cfg(not(feature = "bellman"))]
                ProofAction::CpSetup => panic!("Missing feature: bellman"),
                #[cfg(feature = "spartan")]
                ProofAction::SpartanSetup => {
                    write_data::<_, _>(prover_key, verifier_key, &prover_data, &verifier_data)
                        .unwrap();
                }
                #[cfg(not(feature = "spartan"))]
                ProofAction::SpartanSetup => panic!("Missing feature: spartan"),
            }
        }
        #[cfg(not(feature = "r1cs"))]
        Backend::R1cs { .. } => {
            panic!("Missing feature: r1cs");
        }
    }
}
