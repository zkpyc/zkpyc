//! ZKInterface export tools

pub mod setup;

use crate::utilities::scalar_fields::PrimeField;
// use ff::PrimeField;

use circ::ir::term::Value;
use circ::ir::term::Value::Field;
use circ::cfg::cfg;
// use circ::target::r1cs::wit_comp::{StagedWitCompEvaluator, self};

use crate::utilities::{r1cs::{ProverData, VerifierData, R1csFinal, Var, Lc, VarType},
    wit_comp::{StagedWitCompEvaluator, self}};
use fxhash::FxHasher;
use zkinterface::{
    ConstraintSystem,
    Witness,
    Variables,
    KeyValue,
    StatementBuilder,
    Sink,
    WorkspaceSink,
    BilinearConstraint};
use std::hash::BuildHasherDefault;
use std::{io::Write, collections::HashMap};
use std::path::Path;
use std::mem;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use std::fs::remove_dir_all;


// // Redefinitions from CirC IR, necessary because some fields are private
// mod serde_vk {
//     use bellman::groth16::VerifyingKey;
//     use pairing::Engine;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};

//     pub fn serialize<S: Serializer, E: Engine>(
//         p: &VerifyingKey<E>,
//         ser: S,
//     ) -> Result<S::Ok, S::Error> {
//         let mut bs: Vec<u8> = Vec::new();
//         p.write(&mut bs).unwrap();
//         serde_bytes::ByteBuf::from(bs).serialize(ser)
//     }

//     pub fn deserialize<'de, D: Deserializer<'de>, E: Engine>(
//         de: D,
//     ) -> Result<VerifyingKey<E>, D::Error> {
//         let bs: serde_bytes::ByteBuf = Deserialize::deserialize(de)?;
//         Ok(VerifyingKey::read(&**bs).unwrap())
//     }
// }

// mod serde_pk {
//     use bellman::groth16::Parameters;
//     use pairing::Engine;
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};

//     pub fn serialize<S: Serializer, E: Engine>(
//         p: &Parameters<E>,
//         ser: S,
//     ) -> Result<S::Ok, S::Error> {
//         let mut bs: Vec<u8> = Vec::new();
//         p.write(&mut bs).unwrap();
//         serde_bytes::ByteBuf::from(bs).serialize(ser)
//     }

//     pub fn deserialize<'de, D: Deserializer<'de>, E: Engine>(
//         de: D,
//     ) -> Result<Parameters<E>, D::Error> {
//         let bs: serde_bytes::ByteBuf = Deserialize::deserialize(de)?;
//         Ok(Parameters::read(&**bs, false).unwrap())
//     }
// }

/// Relation-related data that a prover needs to make a proof.
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ProverData {
//     /// R1cs
//     pub r1cs: R1csFinal,
//     /// Witness computation
//     pub precompute: circ::target::r1cs::wit_comp::StagedWitComp,
// }

// #[derive(Serialize, Deserialize)]
// pub struct ProvingKey<E: Engine>(
//     pub ProverData,
//     #[serde(with = "serde_pk")] bellman::groth16::Parameters<E>,
// );

// #[derive(Serialize, Deserialize)]
// pub struct VerifyingKey<E: Engine>(
//     pub VerifierData,
//     #[serde(with = "serde_vk")] bellman::groth16::VerifyingKey<E>,
// );

// /// Convert a (rug) integer to a prime field element.
// pub fn int_to_ff<F: PrimeField>(i: Integer) -> F {
//     let mut accumulator = F::from(0);
//     let limb_bits = (std::mem::size_of::<gmp_mpfr_sys::gmp::limb_t>() as u64) << 3;
//     let limb_base = F::from(2).pow_vartime([limb_bits]);
//     // as_ref yeilds a least-significant-first array.
//     for digit in i.as_ref().iter().rev() {
//         accumulator *= limb_base;
//         accumulator += F::from(*digit);
//     }
//     accumulator
// }

#[derive(PartialEq, Copy, Clone)]
pub enum Target {
    /// Generate constraints, public inputs, witness.
    Prover,
    /// Generate constraints, public inputs.
    Verifier,
    /// Generate constraints only.
    Preprocessing,
}

pub struct ZkifCS<F: PrimeField> {
    pub constraints_per_message: usize,
    statement: StatementBuilder<WorkspaceSink>,
    constraints: ConstraintSystem,
    target: Target,
    witness_ids: Vec<u64>,
    witness_encoding: Vec<u8>,
    phantom: PhantomData<F>,
}

pub struct ZkifWitnesses<F: PrimeField> {
    statement: StatementBuilder<WorkspaceSink>,
    witness_ids: Vec<u64>,
    witness_encoding: Vec<u8>,
    phantom: PhantomData<F>,
}

pub struct ZkifCircuit<F: PrimeField> {
    statement: StatementBuilder<WorkspaceSink>,
    instance_ids: Vec<u64>,
    instance_encoding: Vec<u8>,
    free_variable_id: u64,
    phantom: PhantomData<F>,
}

const DEFAULT_CONSTRAINTS_PER_MESSAGE: usize = 100000;

impl<F: PrimeField> ZkifCS<F> {
    /// Must call finish() to finalize the files in the workspace.
    pub fn new(workspace: impl AsRef<Path>, target: Target) -> Self {
        let sink = WorkspaceSink::new(workspace).unwrap();
        let statement = StatementBuilder::new(sink);

        ZkifCS {
            constraints_per_message: DEFAULT_CONSTRAINTS_PER_MESSAGE,
            statement,
            constraints: ConstraintSystem::default(),
            target,
            witness_ids: vec![],
            witness_encoding: vec![],
            phantom: PhantomData,
        }
    }

    pub fn finish(mut self, name: &str) -> zkinterface::Result<()> {
        if self.constraints.constraints.len() > 0 {
            self.statement.push_constraints(self.constraints)?;
        }

        if self.target == Target::Prover {
            let wit = Witness {
                assigned_variables: Variables {
                    variable_ids: self.witness_ids,
                    values: Some(self.witness_encoding.clone()),
                }
            };
            println!("{:#?}", wit);
            self.statement.push_witness(wit)?;
        }

        let negative_one = F::one().neg();

        // let t: [u8; 32] = negative_one.to_repr().as_ref().try_into().expect("Conversion from Bls12_381Repr to [u8; 32] failed.");
        // for &b in t.iter().rev() {
        //     println!("{:02x}", b);
        // }
        let mut field_maximum = Vec::<u8>::new();
        write_scalar(&negative_one, &mut field_maximum);

        self.statement.header.field_maximum = Some(field_maximum);
        self.statement.header.configuration = Some(vec![
            KeyValue {
                key: "name".to_string(),
                text: Some(name.to_string()),
                data: None,
                number: 0,
            }]);
        self.statement.finish_header()
    }

    fn push_constraint(&mut self, co: BilinearConstraint) -> zkinterface::Result<()> {
        self.constraints.constraints.push(co);

        if self.constraints.constraints.len() >= self.constraints_per_message {
            let cs = mem::replace(&mut self.constraints, ConstraintSystem::default());
            self.statement.push_constraints(cs)?;
        }
        Ok(())
    }
}

impl<F: PrimeField> ZkifWitnesses<F> {
    pub fn new(workspace: impl AsRef<Path>, witness_ids: Vec<u64>, witness_encoding: Vec<u8>) -> Self {
        let sink = WorkspaceSink::new(workspace).unwrap();
        let statement = StatementBuilder::new(sink);

        ZkifWitnesses { 
            statement,
            witness_ids,
            witness_encoding,
            phantom: PhantomData,
        }
    }

    pub fn finish(mut self, name: &str) -> zkinterface::Result<()> {
        let wit = Witness {
            assigned_variables: Variables {
                variable_ids: self.witness_ids,
                values: Some(self.witness_encoding.clone()),
            }
        };
        self.statement.push_witness(wit)

        // let negative_one = F::one().neg();
        // let mut field_maximum = Vec::<u8>::new();
        // write_scalar(&negative_one, &mut field_maximum);

        // self.statement.header.field_maximum = Some(field_maximum);
        // self.statement.header.configuration = Some(vec![
        //     KeyValue {
        //         key: "name".to_string(),
        //         text: Some(name.to_string()),
        //         data: None,
        //         number: 0,
        //     }]);
        // self.statement.finish_header()
    }


}

impl <F: PrimeField> ZkifCircuit<F> {
    pub fn new(workspace: impl AsRef<Path>, instance_ids: Vec<u64>, free_variable_id: u64) -> Self {
        let sink = WorkspaceSink::new(workspace).unwrap();
        let statement = StatementBuilder::new(sink);

        ZkifCircuit { 
            statement,
            instance_ids,
            instance_encoding: vec![],
            free_variable_id,
            phantom: PhantomData,
        }
    }

    pub fn finish(mut self, name: &str) -> zkinterface::Result<()> {
        let connections = Variables {
            variable_ids: self.instance_ids,
            values: Some(self.instance_encoding.clone()),
        };
        
        self.statement.header.instance_variables = connections;
        self.statement.header.free_variable_id = self.free_variable_id;

        let negative_one = F::one().neg();
        let mut field_maximum = Vec::<u8>::new();
        write_scalar(&negative_one, &mut field_maximum);

        self.statement.header.field_maximum = Some(field_maximum);
        self.statement.header.configuration = Some(vec![
            KeyValue {
                key: "name".to_string(),
                text: Some(name.to_string()),
                data: None,
                number: 0,
            }]);
        self.statement.finish_header()

    }
}

pub fn to_zkif_constraint<F: PrimeField>(
    vars: &Vec<Var>,
    a: &Lc,
    b: &Lc,
    c: &Lc,
) -> BilinearConstraint {
    BilinearConstraint {
        linear_combination_a: to_zkif_lc::<F>(vars, a),
        linear_combination_b: to_zkif_lc::<F>(vars, b),
        linear_combination_c: to_zkif_lc::<F>(vars, c),
    }
}

pub fn to_zkif_lc<F: PrimeField>(
    vars: &Vec<Var>,
    lc: &Lc,
) -> Variables {
    let mut variable_ids = Vec::<u64>::new();
    let mut coeffs = Vec::<u8>::new();
    if !lc.constant.is_zero() {
        variable_ids.push(0); // var_0 is always the constant
        write_scalar(&F::int_to_ff((&lc.constant).into()), &mut coeffs);
    }
    for (var, coeff) in &lc.monomials {
        if !coeff.is_zero() {
            let zkid = vars.iter().position(|x| x == var).unwrap() + 1;
            variable_ids.push(zkid as u64);
            write_scalar(&F::int_to_ff((coeff).into()), &mut coeffs);
        }
    }

    Variables { variable_ids, values: Some(coeffs) }
}

pub fn to_zkif_witness<F: PrimeField>(
    values: &Vec<Value>,
) -> Vec<u8> {
    let mut witnesses = Vec::<u8>::new();
    for value in values {
        // Values must always be of Field variant,
        // otherwise something went wrong.
        write_scalar(&F::int_to_ff(value.as_pf().into()), &mut witnesses);
    }
    witnesses
}

pub fn write_scalar<F: PrimeField>(
    fr: &F,
    writer: &mut impl Write,
) {
    let repr = fr.to_repr();
    writer.write_all(repr.as_ref()).unwrap();
}

pub fn write_constraints<F: PrimeField>(
    r1cs: &R1csFinal,
    witness_map: HashMap<String, Value, BuildHasherDefault<FxHasher>>,
) {
    let dir = Path::new("local/test/");
    // let _ = remove_dir_all(dir);

    let mut cs = ZkifCS::<F>::new(dir, Target::Preprocessing);
    // cs.constraints_per_message = 4;
    
    let vars = &r1cs.vars;
    for (i, (a, b, c)) in r1cs.constraints.iter().enumerate() {
        let lc = BilinearConstraint {
            linear_combination_a: to_zkif_lc::<F>(vars, a),
            linear_combination_b: to_zkif_lc::<F>(vars, b),
            linear_combination_c: to_zkif_lc::<F>(vars, c),
        };
        cs.push_constraint(lc).unwrap();

    }

    cs.finish("test").unwrap();
}

pub fn write_assignment<F: PrimeField>(
    first_local_id: u64,
    local_values: &[Value],
) {
    let dir = Path::new("local/test/");
    // let _ = remove_dir_all(dir);

    let mut ids = vec![];
    let mut values = vec![];

    for i in 0..local_values.len() {
        ids.push(first_local_id + i as u64);
        // Values are always prime field elements
        write_scalar(&F::int_to_ff(local_values[i].as_pf().into()), &mut values);
    }

    let witt = ZkifWitnesses::<F>::new(dir, ids, values);
    witt.finish("witt_out").unwrap();
}

pub fn write_circuit<F: PrimeField>(
    first_local_id: u64,
    free_variable_id: u64,
    public_inputs: Option<&[Value]>,
    r1cs_generation: bool,
) {
    let dir = Path::new("local/test/");
    // let _ = remove_dir_all(dir);

    // we do not include the constant one
    let ids = (1..first_local_id).collect();
    // Convert element representations.
    let values = public_inputs.map(|public_inputs| {
        assert_eq!(public_inputs.len() as u64, first_local_id);
        let mut values = vec![];
        for value in public_inputs.iter().skip(1) {
            write_scalar(&F::int_to_ff(value.as_pf().into()), &mut values);
        }
        values
    });

    let mut circuit = ZkifCircuit::<F>::new(dir, ids, free_variable_id);
    circuit.instance_encoding = values.unwrap();

    circuit.finish("circuit").unwrap();
}

pub fn prepare_generate_proof<F: PrimeField>(
    pd: &ProverData,
    witness_map: HashMap<String, Value, BuildHasherDefault<FxHasher>>,
) -> (Vec<Value>, Vec<Value>) {
    let public_variables_count = &pd.r1cs.vars
        .iter()
        .filter(|&var| {
            match var.ty() {
                VarType::Inst => true,
                _ => false,
            }
        })
        .count();

    // Evaluate the witnesses
    let mut evaluator = StagedWitCompEvaluator::new(&pd.precompute);
    let mut ffs = Vec::new();
    ffs.extend(evaluator.eval_stage(witness_map.clone()).into_iter().cloned());
    ffs.extend(
        evaluator
            .eval_stage(Default::default())
            .into_iter()
            .cloned(),
    );
    let mut witness: Vec<Value> = ffs;

    // Insert the one variable assignment
    witness.insert(0, Value::Field(cfg().field().new_v(1)));

    // split witness into public and private inputs at offset
    let mut public_inputs: Vec<Value> = witness.clone();
    let private_inputs: Vec<Value> = public_inputs.split_off(*public_variables_count + 1);

    (
        public_inputs,
        private_inputs,
    )
}

pub fn prepare_verify_proof<F: PrimeField>(
    vd: &VerifierData,
    witness_map: HashMap<String, Value, BuildHasherDefault<FxHasher>>,
) -> (Vec<Value>, u64, u64) {
    let insance_map = vd.eval(&witness_map);
    let mut public_inputs: Vec<Value> = insance_map
        .into_iter()
        .map(|i| Value::from(Field(i)))
        .collect();
    
    let private_variables_count = &vd.r1cs.vars
    .iter()
    .filter(|&var| {
        match var.ty() {
            VarType::FinalWit => true,
            _ => false,
        }
    })
    .count();

    // Insert the one variable assignment
    public_inputs.insert(0, Value::Field(cfg().field().new_v(1)));

    let first_local_id = public_inputs.len() as u64;
    let free_variable_id = first_local_id + *private_variables_count as u64;

    (public_inputs, first_local_id, free_variable_id)
}