use bellman::{
    ConstraintSystem,
    LinearCombination,
    SynthesisError,
    Variable,
    gadgets::num::AllocatedNum,
};
use std::collections::HashMap;
use zkinterface::{
    CircuitHeader, Variables, Result,
    consumers::reader::{Reader, Constraint, Term},
};
use crate::export::write_scalar;
use ff::PrimeField;


/// Convert zkInterface little-endian bytes to bellman Fr.
/// TODO: Verify that Scalar::Repr is little-endian.
pub fn read_scalar<Scalar: PrimeField>(
    encoded: &[u8],
) -> Scalar {
    let mut repr = Scalar::Repr::default();

    {
        let repr: &mut [u8] = repr.as_mut();
        assert!(encoded.len() <= repr.len(), "Element is too big ({} > {} bytes)", encoded.len(), repr.len());
        for i in 0..encoded.len() {
            repr[i] = encoded[i];
        }
    }

    Scalar::from_repr(repr).unwrap()
}

/// Convert zkInterface terms to bellman LinearCombination.
pub fn terms_to_lc<Scalar: PrimeField>(
    vars: &HashMap<u64, Variable>,
    terms: &[Term],
) -> LinearCombination<Scalar> {
    let mut lc = LinearCombination::zero();
    for term in terms {
        let coeff = read_scalar(term.value);
        let var = vars.get(&term.id).unwrap().clone();
        lc = lc + (coeff, var);
    }
    lc
}

/// Enforce a zkInterface constraint in bellman CS.
pub fn enforce<Scalar: PrimeField, CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    vars: &HashMap<u64, Variable>,
    constraint: &Constraint,
) {
    cs.enforce(|| "",
               |_| terms_to_lc(vars, &constraint.a),
               |_| terms_to_lc(vars, &constraint.b),
               |_| terms_to_lc(vars, &constraint.c),
    );
}

/// Call a foreign gadget through zkInterface.
pub fn call_gadget<Scalar: PrimeField, CS: ConstraintSystem<Scalar>>(
    cs: &mut CS,
    inputs: &[AllocatedNum<Scalar>],
    exec_fn: &dyn Fn(&[u8]) -> Result<Reader>,
) -> Result<Vec<AllocatedNum<Scalar>>> {
    let witness_generation = inputs.len() > 0 && inputs[0].get_value().is_some();

    // Serialize input values.
    let values = if witness_generation {
        let mut values = Vec::<u8>::new();
        for i in inputs {
            let val = i.get_value().unwrap();
            write_scalar(&val, &mut values);
        }
        Some(values)
    } else {
        None
    };

    // Describe the input variables.
    let first_input_id = 1;
    let free_variable_id = first_input_id + inputs.len() as u64;

    let call_header = CircuitHeader {
        instance_variables: Variables {
            variable_ids: (first_input_id..free_variable_id).collect(),
            values,
        },
        free_variable_id,
        field_maximum: None,
        configuration: None,
    };

    // Prepare the call.
    let mut call_buf = vec![];
    call_header.write_into(&mut call_buf)?;

    // Call.
    let response = exec_fn(&call_buf).or(Err(SynthesisError::Unsatisfiable))?;

    // Track variables by id. Used to convert constraints.
    let mut id_to_var = HashMap::<u64, Variable>::new();

    id_to_var.insert(0, CS::one());

    for i in 0..inputs.len() {
        id_to_var.insert(call_header.instance_variables.variable_ids[i], inputs[i].get_variable());
    }

    // Collect output variables and values to return.
    let mut outputs = Vec::new();

    // Allocate outputs, with optional values.
    if let Some(output_vars) = response.instance_variables() {
        for var in output_vars {
            let num = AllocatedNum::alloc(
                cs.namespace(|| format!("output_{}", var.id)), || {
                    Ok(read_scalar(var.value))
                })?;

            // Track output variable.
            id_to_var.insert(var.id, num.get_variable());
            outputs.push(num);
        }
    }

    // Allocate private variables, with optional values.
    let private_vars = response.private_variables().unwrap();

    for var in private_vars {
        let num = AllocatedNum::alloc(
            cs.namespace(|| format!("local_{}", var.id)), || {
                Ok(read_scalar(var.value))
            })?;

        // Track private variable.
        id_to_var.insert(var.id, num.get_variable());
    };

    // Add gadget constraints.
    for (i, constraint) in response.iter_constraints().enumerate() {
        enforce(&mut cs.namespace(|| format!("constraint_{}", i)), &id_to_var, &constraint);
    }

    Ok(outputs)
}
