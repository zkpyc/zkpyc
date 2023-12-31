#!/usr/bin/env zsh

set -ex

disable -r time

# cargo build --release

BIN=./target/release/zkpyc
ZK_BIN=./target/release/zk

case "$OSTYPE" in 
    darwin*)
        alias measure_time="gtime --format='%e seconds %M kB'"
    ;;
    linux*)
        alias measure_time="time --format='%e seconds %M kB'"
    ;;
esac

# Test r1cs generation
function r1cs_test {
    pypath=$1
    implementation=$2
    custom_field=$3
    if (( $custom_field == 0 ))
    then
        $BIN $pypath r1cs --action setup --proof-impl $implementation
    else
        $BIN --field-custom-modulus $custom_field $pypath r1cs --action setup --proof-impl $implementation
    fi
}

# Test prove workflow, given an example name
function pf_test {
    ex_name=$1
    implementation=$2
    custom_field=$3
    if (( $custom_field == 0 ))
    then
        $ZK_BIN --inputs $ex_name.pin --action prove --proof-impl $implementation
        $ZK_BIN --inputs $ex_name.vin --action verify --proof-impl $implementation
    else
        $ZK_BIN --field-custom-modulus $custom_field --inputs $ex_name.pin --action prove --proof-impl $implementation
        $ZK_BIN --field-custom-modulus $custom_field --inputs $ex_name.vin --action verify --proof-impl $implementation
    fi
    if [ "$implementation" = "groth16" ]
    then
        rm -rf P V pi
    else
        rm -rf P V
    fi
}

# Test both r1cs generation and prove workflow
function r1cs_pf_test {
    ex_name=$1
    implementation=${2:-'groth16'}
    custom_field=${3:-0}
    r1cs_test $ex_name $implementation $custom_field
    pf_test $ex_name $implementation $custom_field
}

# r1cs_pf_test examples/_3_plus_int.py
# r1cs_pf_test examples/arr_cls_arr_cls.py
# r1cs_pf_test examples/arr_cpy.py
# r1cs_pf_test examples/assert.py
# r1cs_pf_test examples/assert2.py
# r1cs_pf_test examples/field_max.py
# r1cs_pf_test examples/isolate_assert.py
# r1cs_pf_test examples/many_cond.py
# r1cs_pf_test examples/many_pub.py
r1cs_pf_test examples/mm.py
# r1cs_pf_test examples/shear_point.py
# r1cs_pf_test examples/from_bits.py
# r1cs_pf_test examples/to_bits.py
# r1cs_pf_test examples/mul.py
# r1cs_pf_test examples/sha_temp1.py
# r1cs_pf_test examples/test_sha256.py zk-interface 7237005577332262213973186563042994240857116359379907606001950938285454250989
# r1cs_pf_test examples/plus_field.py zk-interface
# r1cs_pf_test examples/qeval.py zk-interface 7237005577332262213973186563042994240857116359379907606001950938285454250989
# r1cs_pf_test examples/zkinterface.py zk-interface
# r1cs_pf_test examples/test_pedersen.py zk-interface 21888242871839275222246405745257275088548364400416034343698204186575808495617
# r1cs_pf_test examples/unpack.py zk-interface 21888242871839275222246405745257275088548364400416034343698204186575808495617
# r1cs_pf_test examples/lookup_window.py zk-interface 21888242871839275222246405745257275088548364400416034343698204186575808495617
# r1cs_pf_test examples/to_bits.py zk-interface 21888242871839275222246405745257275088548364400416034343698204186575808495617
# r1cs_pf_test examples/div.py zk-interface 21888242871839275222246405745257275088548364400416034343698204186575808495617
# r1cs_pf_test examples/edwardsAdd.py zk-interface 21888242871839275222246405745257275088548364400416034343698204186575808495617
# r1cs_pf_test examples/plus.py
# r1cs_pf_test examples/pack.py
# r1cs_pf_test examples/sum.py