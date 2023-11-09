use curve25519_dalek::scalar::Scalar as Curve25519;
use ff::Field;
use rug::Integer;
use self::{bn256::Bn256, bls12_381::Bls12_381};


pub mod bls12_381 {
    use ff::PrimeField;

    #[derive(PrimeField)]
    #[PrimeFieldModulus = "52435875175126190479447740508185965837690552500527637822603658699938581184513"]
    #[PrimeFieldGenerator = "7"]
    #[PrimeFieldReprEndianness = "little"]
    pub struct Bls12_381([u64; 4]);
}

pub mod bn256 {
    use ff::PrimeField;

    #[derive(PrimeField)]
    #[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
    #[PrimeFieldGenerator = "5"]
    #[PrimeFieldReprEndianness = "little"]
    pub struct Bn256([u64; 4]);
}

// Ristretto255 has scalar field 7237005577332262213973186563042994240857116359379907606001950938285454250989

pub trait PrimeField {
    type Repr: Copy + Default + Send + Sync + 'static + AsRef<[u8]> + AsMut<[u8]>;

    fn one() -> Self;
    fn neg(&self) -> Self;
    fn to_repr(&self) -> Self::Repr;
    // Tried implementing From<Integer> but only works for local structs
    fn int_to_ff(_: Integer) -> Self;
}

impl PrimeField for Bn256 {
    type Repr = [u8; 32];

    fn one() -> Bn256 {
       <Bn256 as ff::Field>::one()
    }

    fn neg(&self) -> Bn256 {
        <Bn256 as std::ops::Neg>::neg(*self)
    }

    fn to_repr(&self) -> Self::Repr {
        <Bn256 as ff::PrimeField>::to_repr(self).as_ref().try_into().expect("Conversion from Bn256Repr to [u8; 32] failed.")
    }

    fn int_to_ff(value: Integer) -> Self {
        let mut accumulator = Bn256::from(0);
        let limb_bits = (std::mem::size_of::<gmp_mpfr_sys::gmp::limb_t>() as u64) << 3;
        let limb_base = Bn256::from(2).pow_vartime([limb_bits]);
        // as_ref yields a least-significant-first array.
        for digit in value.as_ref().iter().rev() {
            accumulator *= limb_base;
            accumulator += Bn256::from(*digit);
        }
        accumulator
    }

}

impl PrimeField for Bls12_381 {
    type Repr = [u8; 32];

    fn one() -> Bls12_381 {
        <Bls12_381 as ff::Field>::one()
    }

    fn neg(&self) -> Bls12_381 {
        <Bls12_381 as std::ops::Neg>::neg(*self)
    }

    fn to_repr(&self) -> Self::Repr {
        <Bls12_381 as ff::PrimeField>::to_repr(self).as_ref().try_into().expect("Conversion from Bls12_381Repr to [u8; 32] failed.")
    }

    fn int_to_ff(value: Integer) -> Self {
        let mut accumulator = Bls12_381::from(0);
        let limb_bits = (std::mem::size_of::<gmp_mpfr_sys::gmp::limb_t>() as u64) << 3;
        let limb_base = Bls12_381::from(2).pow_vartime([limb_bits]);
        // as_ref yields a least-significant-first array.
        for digit in value.as_ref().iter().rev() {
            accumulator *= limb_base;
            accumulator += Bls12_381::from(*digit);
        }
        accumulator
    }

}

impl PrimeField for Curve25519 {
    type Repr = [u8; 32];

    fn one() -> Curve25519 {
        Curve25519::one()
    }

    fn neg(&self) -> Curve25519 {
        <Curve25519 as std::ops::Neg>::neg(*self)
    }

    fn to_repr(&self) -> Self::Repr {
        self.to_bytes()
    }

    fn int_to_ff(value: Integer) -> Self {
        let mut accumulator = Curve25519::from(0 as u64);
        let limb_bits = (std::mem::size_of::<gmp_mpfr_sys::gmp::limb_t>() as u64) << 3;
        let limb_base = pow_vartime(Curve25519::from(2 as u64), [limb_bits]);
        // as_ref yields a least-significant-first array.
        for digit in value.as_ref().iter().rev() {
            accumulator *= limb_base;
            accumulator += Curve25519::from(*digit);
        }
        accumulator
    }
}

fn pow_vartime<S: AsRef<[u64]>>(curve25519: Curve25519, exp: S) -> Curve25519 {
    let mut res = Curve25519::one();
    for e in exp.as_ref().iter().rev() {
        for i in (0..64).rev() {
            res = res*res;

            if ((*e >> i) & 1) == 1 {
                res *= curve25519;
            }
        }
    }
    res
}

