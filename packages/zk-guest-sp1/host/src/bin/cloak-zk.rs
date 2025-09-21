use anyhow::Result;
use clap::{Parser, Subcommand};
use sp1_sdk::{include_elf, ProverClient, SP1ProofWithPublicValues, SP1Stdin};
use std::fs;

const ELF: &[u8] = include_elf!("zk-guest-sp1-guest");

#[derive(Parser)]
#[command(name = "cloak-zk")]
#[command(about = "Cloak ZK proof generation tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Prove {
        #[arg(short, long)]
        private: String,
        #[arg(short, long)]
        public: String,
        #[arg(short, long)]
        outputs: String,
        #[arg(short, long)]
        proof: String,
        #[arg(short, long)]
        pubout: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove {
            private,
            public,
            outputs,
            proof,
            pubout,
        } => {
            // Setup
            let client = ProverClient::from_env();
            let (pk, _vk) = client.setup(ELF);
            
            // Read input files
            let private_json = fs::read_to_string(&private)?;
            let public_json = fs::read_to_string(&public)?;
            let outputs_json = fs::read_to_string(&outputs)?;
            
            // Create combined input
            let combined_input = format!(
                r#"{{
                    "private": {},
                    "public": {},
                    "outputs": {}
                }}"#,
                private_json, public_json, outputs_json
            );
            
            // Generate proof
            let mut stdin = SP1Stdin::new();
            stdin.write(&combined_input);
            
            let proof_result = client.prove(&pk, &stdin).groth16().run()?;
            
            // Save proof
            proof_result.save(&proof)?;
            
            // Save public inputs
            let sp1_proof_with_public_values = SP1ProofWithPublicValues::load(&proof)?;
            let public_inputs = sp1_proof_with_public_values.public_values.to_vec();
            let public_inputs_len = public_inputs.len();
            fs::write(&pubout, public_inputs)?;
            
            println!("Proof generated successfully!");
            println!("Proof size: {} bytes", sp1_proof_with_public_values.bytes().len());
            println!("Public inputs size: {} bytes", public_inputs_len);
            
            Ok(())
        }
    }
}
