use std::io::Write;
use zkinterface::{Variables, BilinearConstraint};
use bellman::{LinearCombination, Index};
use ff::PrimeField;


pub fn to_zkif_constraint<Scalar: PrimeField>(
    a: LinearCombination<Scalar>,
    b: LinearCombination<Scalar>,
    c: LinearCombination<Scalar>,
) -> BilinearConstraint {
    BilinearConstraint {
        linear_combination_a: to_zkif_lc(a),
        linear_combination_b: to_zkif_lc(b),
        linear_combination_c: to_zkif_lc(c),
    }
}

pub fn to_zkif_lc<Scalar: PrimeField>(
    lc: LinearCombination<Scalar>,
) -> Variables {
    let mut variable_ids = Vec::<u64>::new();
    let mut coeffs = Vec::<u8>::new();

    for (var, coeff) in lc.as_ref() {
        let zkid = match var.get_unchecked() {
            Index::Input(zkid) => zkid,
            Index::Aux(zkid) => zkid,
        };
        variable_ids.push(zkid as u64);

        write_scalar(coeff, &mut coeffs);
    }

    Variables { variable_ids, values: Some(coeffs) }
}

/// Convert bellman Fr to zkInterface little-endian bytes.
/// TODO: Verify that Scalar::Repr is little-endian.
pub fn write_scalar<Scalar: PrimeField>(
    fr: &Scalar,
    writer: &mut impl Write,
) {
    let repr = fr.to_repr();
    writer.write_all(repr.as_ref()).unwrap();
}
