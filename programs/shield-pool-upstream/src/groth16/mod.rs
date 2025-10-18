// Groth16 verifier implementation for Cloak
// Based on light-protocol's groth16-solana verifier
// Uses Solana's alt_bn128 syscalls for BN254 curve operations

use crate::error::ShieldPoolError;
use ark_ff::PrimeField;
use num_bigint::BigUint;
use pinocchio::program_error::ProgramError;
use solana_bn254::prelude::{alt_bn128_addition, alt_bn128_multiplication, alt_bn128_pairing};

#[derive(PartialEq, Eq, Debug)]
pub struct Groth16Verifyingkey<'a> {
    pub nr_pubinputs: usize,
    pub vk_alpha_g1: [u8; 64],
    pub vk_beta_g2: [u8; 128],
    pub vk_gamma_g2: [u8; 128],
    pub vk_delta_g2: [u8; 128],
    pub vk_ic: &'a [[u8; 64]],
}

#[derive(PartialEq, Eq, Debug)]
pub struct Groth16Verifier<'a, const NR_INPUTS: usize> {
    proof_a: &'a [u8; 64],
    proof_b: &'a [u8; 128],
    proof_c: &'a [u8; 64],
    public_inputs: &'a [[u8; 32]; NR_INPUTS],
    prepared_public_inputs: [u8; 64],
    verifyingkey: &'a Groth16Verifyingkey<'a>,
}

impl<const NR_INPUTS: usize> Groth16Verifier<'_, NR_INPUTS> {
    pub fn new<'a>(
        proof_a: &'a [u8; 64],
        proof_b: &'a [u8; 128],
        proof_c: &'a [u8; 64],
        public_inputs: &'a [[u8; 32]; NR_INPUTS],
        verifyingkey: &'a Groth16Verifyingkey<'a>,
    ) -> Result<Groth16Verifier<'a, NR_INPUTS>, ProgramError> {
        if proof_a.len() != 64 {
            return Err(ShieldPoolError::InvalidG1Length.into());
        }

        if proof_b.len() != 128 {
            return Err(ShieldPoolError::InvalidG2Length.into());
        }

        if proof_c.len() != 64 {
            return Err(ShieldPoolError::InvalidG1Length.into());
        }

        if public_inputs.len() + 1 != verifyingkey.vk_ic.len() {
            return Err(ShieldPoolError::InvalidPublicInputsLength.into());
        }

        Ok(Groth16Verifier {
            proof_a,
            proof_b,
            proof_c,
            public_inputs,
            prepared_public_inputs: [0u8; 64],
            verifyingkey,
        })
    }

    pub fn prepare_inputs<const CHECK: bool>(&mut self) -> Result<(), ProgramError> {
        let mut prepared_public_inputs = self.verifyingkey.vk_ic[0];

        for (i, input) in self.public_inputs.iter().enumerate() {
            if CHECK && !is_less_than_bn254_field_size_be(input) {
                return Err(ShieldPoolError::PublicInputGreaterThanFieldSize.into());
            }
            let mul_res = alt_bn128_multiplication(
                &[&self.verifyingkey.vk_ic[i + 1][..], &input[..]].concat(),
            )
            .map_err(|_| ShieldPoolError::PreparingInputsG1MulFailed)?;
            prepared_public_inputs =
                alt_bn128_addition(&[&mul_res[..], &prepared_public_inputs[..]].concat())
                    .map_err(|_| ShieldPoolError::PreparingInputsG1AdditionFailed)?[..]
                    .try_into()
                    .map_err(|_| ShieldPoolError::PreparingInputsG1AdditionFailed)?;
        }

        self.prepared_public_inputs = prepared_public_inputs;

        Ok(())
    }

    /// Verifies the proof, and checks that public inputs are smaller than field size
    pub fn verify(&mut self) -> Result<bool, ProgramError> {
        self.verify_common::<true>()
    }

    /// Verifies the proof, does not check that public inputs are smaller than field size
    pub fn verify_unchecked(&mut self) -> Result<bool, ProgramError> {
        self.verify_common::<false>()
    }

    fn verify_common<const CHECK: bool>(&mut self) -> Result<bool, ProgramError> {
        self.prepare_inputs::<CHECK>()?;

        let pairing_input = [
            self.proof_a.as_slice(),
            self.proof_b.as_slice(),
            self.prepared_public_inputs.as_slice(),
            self.verifyingkey.vk_gamma_g2.as_slice(),
            self.proof_c.as_slice(),
            self.verifyingkey.vk_delta_g2.as_slice(),
            self.verifyingkey.vk_alpha_g1.as_slice(),
            self.verifyingkey.vk_beta_g2.as_slice(),
        ]
        .concat();

        let pairing_res = alt_bn128_pairing(pairing_input.as_slice())
            .map_err(|_| ShieldPoolError::ProofVerificationFailed)?;

        if pairing_res[31] != 1 {
            return Err(ShieldPoolError::ProofVerificationFailed.into());
        }
        Ok(true)
    }
}

pub fn is_less_than_bn254_field_size_be(bytes: &[u8; 32]) -> bool {
    let bigint = BigUint::from_bytes_be(bytes);
    bigint < ark_bn254::Fr::MODULUS.into()
}
