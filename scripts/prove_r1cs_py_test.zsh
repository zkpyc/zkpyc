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
    $BIN $pypath r1cs --action setup --proof-impl groth16
}

# Test prove workflow, given an example name
function pf_test {
    ex_name=$1
    $ZK_BIN --inputs $ex_name.pin --action prove --proof-impl groth16
    $ZK_BIN --inputs $ex_name.vin --action verify --proof-impl groth16
    rm -rf P V pi
}

r1cs_test examples/test_sha256.py
pf_test examples/test_sha256.py