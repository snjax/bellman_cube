#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate bellman;
extern crate rand;

use self::rand::{thread_rng, Rng};
use crate::hasher::BabyPedersenHasher;
use bellman::pairing::ff::{Field, PrimeField, PrimeFieldRepr};
use sapling_crypto::jubjub::{JubjubEngine, FixedGenerators, Unknown, edwards, JubjubParams};
use sapling_crypto::circuit::{Assignment, boolean, ecc, pedersen_hash, blake2s, sha256, num, multipack, baby_eddsa, float_point};
use sapling_crypto::circuit::num::{AllocatedNum, Num};
use sapling_crypto::alt_babyjubjub::{AltJubjubBn256};
use bellman::pairing::{Engine};
use bellman::pairing::bn256::{Bn256, Fr};
use bellman::{Circuit, ConstraintSystem, SynthesisError};
use bellman::groth16::{Proof, generate_random_parameters, prepare_verifying_key, create_random_proof, verify_proof};

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

// #[test]
pub fn test_pedersen_proof(){
    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut thread_rng();
    let pedersen_params = &AltJubjubBn256::new();

    let preimage = Fr::from_hex("0x01").unwrap();
    let hasher = BabyPedersenHasher::default();
    let hash = hasher.hash(preimage);
    
    println!("Creating parameters...");
    
    // Create parameters for our circuit
    let params = {
        let c = PedersenDemo::<Bn256> {
            params: pedersen_params,
            hash: None,
            preimage: None
        };

        generate_random_parameters(c, rng).unwrap()
    };
    
    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);

    println!("Creating proofs...");
    
    // Create an instance of circuit
    let c = PedersenDemo::<Bn256> {
        params: pedersen_params,
        hash: Some(hash),
        preimage: Some(preimage)
    };
    
    // Create a groth16 proof with our parameters.
    let proof = create_random_proof(c, &params, rng).unwrap();

    let result = verify_proof(
        &pvk,
        &proof,
        &[
            hash,
            preimage
        ]
    ).unwrap();

    println!("Success: {}", result);
}
