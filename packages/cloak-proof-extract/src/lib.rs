#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std", test))]
extern crate alloc;

use core::convert::TryInto;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidFormat,
}

impl Error {
    #[inline]
    fn invalid() -> Self { Error::InvalidFormat }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidFormat => write!(f, "invalid proof/public inputs format"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Extract the 260-byte Groth16 proof fragment from an SP1 proof bundle (bincode of SP1ProofWithPublicValues).
///
/// Strategy: scan for bincode-like Vec<u8> length prefix (u64 LE == 260) followed by 260 bytes.
/// Select the first reasonable candidate that isn't all zeros. If none, return InvalidFormat.
pub fn extract_groth16_260(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], Error> {
    // Heuristic 1: known stable offset observed in current SP1 bundles
    // (proof bytes start at 0x2b0 and span 260 bytes)
    const KNOWN_OFF: usize = 0x2b0;
    const WANT_USIZE: usize = 260;
    if sp1_proof_bundle.len() >= KNOWN_OFF + WANT_USIZE {
        let slice = &sp1_proof_bundle[KNOWN_OFF..KNOWN_OFF + WANT_USIZE];
        if slice.iter().any(|&b| b != 0) {
            return slice.try_into().map_err(|_| Error::invalid());
        }
    }

    // Heuristic 2: scan for bincode-like u64 length prefix == 260
    const WANT: u64 = 260;
    if sp1_proof_bundle.len() >= 8 + WANT as usize {
        let max = sp1_proof_bundle.len() - 8;
        let mut i = 0usize;
        while i <= max {
            let len = u64::from_le_bytes([
                sp1_proof_bundle[i],
                sp1_proof_bundle[i + 1],
                sp1_proof_bundle[i + 2],
                sp1_proof_bundle[i + 3],
                sp1_proof_bundle[i + 4],
                sp1_proof_bundle[i + 5],
                sp1_proof_bundle[i + 6],
                sp1_proof_bundle[i + 7],
            ]);
            if len == WANT {
                let start = i + 8;
                let end = start + WANT as usize;
                if end <= sp1_proof_bundle.len() {
                    let slice = &sp1_proof_bundle[start..end];
                    let nonzero = slice.iter().filter(|&&b| b != 0).count();
                    if nonzero >= 8 {
                        return slice.try_into().map_err(|_| Error::invalid());
                    }
                }
            }
            i += 1;
        }
    }

    Err(Error::invalid())
}

/// 104-byte public inputs structure
#[cfg_attr(feature = "hex", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicInputs {
    #[cfg_attr(feature = "hex", serde(with = "hex32_serde"))]
    pub root: [u8; 32],
    #[cfg_attr(feature = "hex", serde(with = "hex32_serde"))]
    pub nf: [u8; 32],
    #[cfg_attr(feature = "hex", serde(with = "hex32_serde"))]
    pub outputs_hash: [u8; 32],
    pub amount: u64, // little-endian in the 104-byte payload
}

/// Parse 104-byte public inputs (root||nf||outputs_hash||amount_le)
pub fn parse_public_inputs_104(bytes: &[u8]) -> Result<PublicInputs, Error> {
    if bytes.len() != 104 {
        return Err(Error::invalid());
    }
    let root: [u8; 32] = bytes[0..32].try_into().map_err(|_| Error::invalid())?;
    let nf: [u8; 32] = bytes[32..64].try_into().map_err(|_| Error::invalid())?;
    let outputs_hash: [u8; 32] = bytes[64..96].try_into().map_err(|_| Error::invalid())?;
    let amount = u64::from_le_bytes(bytes[96..104].try_into().map_err(|_| Error::invalid())?);

    Ok(PublicInputs { root, nf, outputs_hash, amount })
}

/// Optional SP1-backed helpers (requires feature = "sp1")
#[cfg(feature = "sp1")]
mod sp1_helpers {
    use super::*;
    use bincode;
    use sp1_sdk::SP1ProofWithPublicValues;

    /// Deserialize an SP1 proof bundle via bincode and return the 260-byte Groth16 proof bytes.
    pub fn extract_groth16_260_sp1(sp1_proof_bundle: &[u8]) -> Result<[u8; 260], Error> {
        let proof: SP1ProofWithPublicValues =
            bincode::deserialize(sp1_proof_bundle).map_err(|_| Error::InvalidFormat)?;
        let bytes = proof.bytes();
        if bytes.len() != 260 {
            return Err(Error::InvalidFormat);
        }
        let mut out = [0u8; 260];
        out.copy_from_slice(bytes);
        Ok(out)
    }

    /// Deserialize SP1 bundle and extract raw 104-byte public inputs committed by the guest.
    pub fn extract_public_inputs_104_sp1(sp1_proof_bundle: &[u8]) -> Result<[u8; 104], Error> {
        let proof: SP1ProofWithPublicValues =
            bincode::deserialize(sp1_proof_bundle).map_err(|_| Error::InvalidFormat)?;
        let v = proof.public_values.to_vec();
        if v.len() < 104 {
            return Err(Error::InvalidFormat);
        }
        let mut out = [0u8; 104];
        out.copy_from_slice(&v[..104]);
        Ok(out)
    }

    /// Deserialize SP1 bundle and parse canonical PublicInputs.
    pub fn parse_public_inputs_104_sp1(sp1_proof_bundle: &[u8]) -> Result<PublicInputs, Error> {
        let raw = extract_public_inputs_104_sp1(sp1_proof_bundle)?;
        super::parse_public_inputs_104(&raw)
    }
}

#[cfg(feature = "sp1")]
pub use sp1_helpers::{extract_groth16_260_sp1, extract_public_inputs_104_sp1, parse_public_inputs_104_sp1};

// serde helpers for hex feature
#[cfg(feature = "hex")]
mod hex32_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(v: &[u8; 32], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&hex::encode(v))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        if s.len() != 64 {
            return Err(serde::de::Error::custom("expected 64 hex chars"));
        }
        let mut out = [0u8; 32];
        let bytes = hex::decode(&s).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn locate(path_candidates: &[&str]) -> Option<PathBuf> {
        for p in path_candidates {
            let pb = PathBuf::from(p);
            if pb.exists() { return Some(pb); }
        }
        None
    }

    #[test]
    fn test_parse_public_inputs_from_out_file() {
        let public_path = locate(&[
            "../zk-guest-sp1/out/public.bin",
            "../../zk-guest-sp1/out/public.bin",
            "../../packages/zk-guest-sp1/out/public.bin",
            "../../../packages/zk-guest-sp1/out/public.bin",
        ]).expect("public.bin not found in expected locations");

        let buf = fs::read(public_path).expect("read public.bin");
        assert_eq!(buf.len(), 104, "public.bin must be 104 bytes");
        let pi = parse_public_inputs_104(&buf).expect("parse 104 public inputs");
        // basic sanity checks
        assert!(pi.root.iter().any(|&b| b != 0));
        assert!(pi.outputs_hash.iter().any(|&b| b != 0));
        // amount parses as u64
        let _amt: u64 = pi.amount;
    }

    #[test]
    fn test_extract_groth16_from_proof_bundle() {
        // Try multiple relative paths so tests work from workspace
        let proof_path = locate(&[
            "../zk-guest-sp1/out/proof.bin",
            "../../zk-guest-sp1/out/proof.bin",
            "../../packages/zk-guest-sp1/out/proof.bin",
            "../../../packages/zk-guest-sp1/out/proof.bin",
        ]).expect("proof.bin not found in expected locations");

        let bundle = fs::read(proof_path).expect("read proof.bin");
        let frag = extract_groth16_260(&bundle).expect("extract 260-byte groth16 fragment");
        assert_eq!(frag.len(), 260);
        // not all zeros
        assert!(frag.iter().any(|&b| b != 0));
    }
}
