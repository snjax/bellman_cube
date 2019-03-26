#![allow(unused_imports)]
extern crate bellman;
extern crate rand;

mod hasher;
mod bit_iterator;
use crate::hasher::BabyPedersenHasher;

use rand::{thread_rng, Rng};
use bellman::pairing::ff::{Field, PrimeField, PrimeFieldRepr};
use sapling_crypto::jubjub::{JubjubEngine, JubjubParams};
use sapling_crypto::circuit::{Assignment, boolean, ecc, pedersen_hash, blake2s, sha256, num, multipack, baby_eddsa, float_point};
use sapling_crypto::circuit::num::{AllocatedNum, Num};
use sapling_crypto::alt_babyjubjub::{AltJubjubBn256};
use bellman::pairing::bn256::{Bn256, Fr};
use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bellman::groth16::{Proof, generate_random_parameters, prepare_verifying_key, create_random_proof, verify_proof};
use sapling_crypto::circuit::test::TestConstraintSystem;

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
            |lc| lc + hash.get_variable()
        );
        Ok(())
    }
}

// #[test]
pub fn test_pedersen_proof(){
    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut thread_rng();
    let pedersen_params = &AltJubjubBn256::new();

    let preimage = Fr::from_hex("0x0b").unwrap();
    let hasher = BabyPedersenHasher::default();
    let hash = hasher.hash(preimage);

    println!("Creating parameters...");
    let params = {
        let c = PedersenDemo::<Bn256> {
            params: pedersen_params,
            hash: None,
            preimage: None
        };
        generate_random_parameters(c, rng).unwrap()
    };

    // Prepare the verification key (for proof verification)
    let vk = prepare_verifying_key(&params.vk);

    println!("Checking constraints...");
    let c = PedersenDemo::<Bn256> {
        params: pedersen_params,
        hash: Some(hash.clone()),
        preimage: Some(preimage.clone())
    };
    let mut cs = TestConstraintSystem::<Bn256>::new();
    c.synthesize(&mut cs).unwrap();
    println!("Unconstrained: {}", cs.find_unconstrained());
    let err = cs.which_is_unsatisfied();
    if err.is_some() {
        panic!("ERROR satisfying in: {}", err.unwrap());
    }

    println!("Creating proofs...");
    let c = PedersenDemo::<Bn256> {
        params: pedersen_params,
        hash: Some(hash),
        preimage: Some(preimage)
    };
    let stopwatch = std::time::Instant::now();
    let proof = create_random_proof(c, &params, rng).unwrap();
    println!("Proof time: {}ms", stopwatch.elapsed().as_millis());

    let result = verify_proof(
        &vk,
        &proof,
        &[
            hash,
            preimage
        ]
    ).unwrap();
    assert!(result, "Proof is correct");
}

fn main() {
    test_pedersen_proof();
    println!("Done");
}