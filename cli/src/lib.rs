use std::{fs, path::Path};

use anyhow::Result;
use ark_serialize::CanonicalDeserialize;
use liminal_ark_relations::{
    environment::{CircuitField, Groth16, ProvingSystem},
    serialization::serialize,
    ConstraintSynthesizer,
};

pub mod application;
pub mod bright_disputes;
pub mod bright_disputes_ink;
pub mod helpers;

pub fn generate_proof(
    circuit: impl ConstraintSynthesizer<CircuitField>,
    proving_key_file: &Path,
) -> Result<Vec<u8>> {
    let pk_bytes = fs::read(proving_key_file)?;
    let pk = <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?;
    Ok(serialize(&Groth16::prove(&pk, circuit)))
}
