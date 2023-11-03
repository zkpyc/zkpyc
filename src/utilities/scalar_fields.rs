use std::fmt;

use curve25519_dalek::scalar::Scalar as Ed25519;
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
        // as_ref yeilds a least-significant-first array.
        for digit in value.as_ref().iter().rev() {
            accumulator *= limb_base;
            accumulator += Bn256::from(*digit);
        }
        accumulator
    }

}

// impl fmt::Debug for Bn256 {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let tmp = self.to_bytes();
//         write!(f, "0x")?;
//         for &b in tmp.iter().rev() {
//             write!(f, "{:02x}", b)?;
//         }
//         Ok(())
//     }
// }

// impl fmt::Display for Bn256 {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }


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
        // as_ref yeilds a least-significant-first array.
        for digit in value.as_ref().iter().rev() {
            accumulator *= limb_base;
            accumulator += Bls12_381::from(*digit);
        }
        accumulator
    }

}


impl PrimeField for Ed25519 {
    type Repr = [u8; 32];

    fn one() -> Ed25519 {
        Ed25519::one()
    }

    fn neg(&self) -> Ed25519 {
        <Ed25519 as std::ops::Neg>::neg(*self)
    }

    fn to_repr(&self) -> Self::Repr {
        self.to_bytes()
    }

    fn int_to_ff(value: Integer) -> Self {
        let mut accumulator = Ed25519::from(0 as u64);
        let limb_bits = (std::mem::size_of::<gmp_mpfr_sys::gmp::limb_t>() as u64) << 3;
        let limb_base = pow_vartime(Ed25519::from(2 as u64), [limb_bits]);
        // as_ref yeilds a least-significant-first array.
        for digit in value.as_ref().iter().rev() {
            accumulator *= limb_base;
            accumulator += Ed25519::from(*digit);
        }
        accumulator
    }

}

fn pow_vartime<S: AsRef<[u64]>>(ed25519: Ed25519, exp: S) -> Ed25519 {
    let mut res = Ed25519::one();
    for e in exp.as_ref().iter().rev() {
        for i in (0..64).rev() {
            res = res*res;

            if ((*e >> i) & 1) == 1 {
                res *= ed25519;
            }
        }
    }

    res
}

