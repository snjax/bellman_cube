#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate bellman;
extern crate pairing;
extern crate rand;

// For randomness (during paramgen and proof generation)
use self::rand::{thread_rng, Rng};

// Bring in some tools for using pairing-friendly curves
use ff::{
    Field,
    PrimeField,
    PrimeFieldRepr
};

use sapling_crypto::jubjub::{
    JubjubEngine,
    FixedGenerators,
    Unknown,
    edwards,
    JubjubParams
};


use sapling_crypto::circuit::{
    Assignment,
    boolean,
    ecc,
    pedersen_hash,
    blake2s,
    sha256,
    num,
    multipack,
    baby_eddsa,
    float_point,
};

use sapling_crypto::circuit::num::{AllocatedNum, Num};

use pairing::{
    Engine
};

// We're going to use the bn256 pairing-friendly elliptic curve.
use pairing::bn256::{
    Bn256,
    Fr
};

// We'll use these interfaces to construct our circuit.
use bellman::{
    Circuit,
    ConstraintSystem,
    SynthesisError
};

// We're going to use the Groth16 proving system.
use bellman::groth16::{
    Proof,
    generate_random_parameters,
    prepare_verifying_key,
    create_random_proof,
    verify_proof,
};

#[derive(Clone)]
pub struct PedersenDemo<'a, E: JubjubEngine> {
    pub params: &'a E::Params,
    pub hash: Option<E::Fr>,
    pub preimage: Option<E::Fr>
}

impl <'a, E: JubjubEngine> Circuit<E> for PedersenDemo<'a, E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self, 
        cs: &mut CS
    ) -> Result<(), SynthesisError>
    {


        let hash = AllocatedNum::alloc(
            cs.namespace(|| "hash"),
            || {
                let hash_value = self.hash;
                Ok(*hash_value.get()?)
            }
        )?;
        hash.inputize(cs.namespace(|| "hash input"))?;


        let preimage = AllocatedNum::alloc(
            cs.namespace(|| "preimage"),
            || {
                let preimage_value = self.preimage;
                Ok(*preimage_value.get()?)
            }
        )?;
        preimage.inputize(cs.namespace(|| "preimage input"))?;


        let preimage_bits = preimage.into_bits_le(cs.namespace(|| "preimage into bits"))?;


        let hash_calculated = pedersen_hash::pedersen_hash(
            cs.namespace(|| "hash calculated"),
            pedersen_hash::Personalization::NoteCommitment,
            &preimage_bits,
            self.params
        )?.get_x().clone();


        cs.enforce(
            || "add constraint between input and pedersen hash output",
            |lc| lc + hash_calculated.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + preimage.get_variable()
        );
        Ok(())
    }
}

