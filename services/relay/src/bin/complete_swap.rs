/// Manual swap completion tool - completes a stuck swap that has WithdrawSwap done but ExecuteSwap pending
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use std::env;
use std::io::BufRead;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <nullifier_hex> [relay_keypair_path]", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!(
            "  {} 650dbce1b64abc09c9ede2421c1c9102ea161db012be12c61f7951b5bbed81c0",
            args[0]
        );
        std::process::exit(1);
    }

    let nullifier_hex = &args[1];
    let relay_keypair_path = if args.len() > 2 {
        args[2].clone()
    } else {
        env::var("RELAY_KEYPAIR_PATH").unwrap_or_else(|_| "admin-keypair.json".to_string())
    };

    let rpc_url =
        env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let program_id_str = env::var("PROGRAM_ID")
        .unwrap_or_else(|_| "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".to_string());

    println!("🔄 Manually Completing Swap");
    println!("   Nullifier: {}", nullifier_hex);
    println!("   Program ID: {}", program_id_str);
    println!("   RPC: {}", rpc_url);
    println!("   Relay Keypair: {}", relay_keypair_path);
    println!();

    // Load relay keypair
    let relay_keypair = read_keypair_file(&relay_keypair_path).map_err(|e| {
        anyhow::anyhow!("Failed to read keypair from {}: {}", relay_keypair_path, e)
    })?;
    let relay_pubkey = relay_keypair.pubkey();
    println!("✅ Relay: {}", relay_pubkey);

    // Decode nullifier
    let nullifier_bytes =
        hex::decode(nullifier_hex).map_err(|e| anyhow::anyhow!("Invalid nullifier hex: {}", e))?;
    if nullifier_bytes.len() != 32 {
        anyhow::bail!("Nullifier must be 32 bytes");
    }
    let mut nullifier = [0u8; 32];
    nullifier.copy_from_slice(&nullifier_bytes);

    // Parse program ID
    let program_id = Pubkey::from_str(&program_id_str)?;

    // Derive SwapState PDA
    let (swap_state_pda, _bump) =
        Pubkey::find_program_address(&[b"swap_state", &nullifier], &program_id);

    println!("📍 SwapState PDA: {}", swap_state_pda);
    println!();

    // Connect to RPC
    let client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    // Load SwapState account
    let swap_state_account = client.get_account(&swap_state_pda).map_err(|e| {
        anyhow::anyhow!(
            "SwapState PDA not found - swap may already be completed: {}",
            e
        )
    })?;

    if swap_state_account.data.len() != 121 {
        anyhow::bail!(
            "Invalid SwapState account size: {}",
            swap_state_account.data.len()
        );
    }

    // Parse SwapState data
    let sol_amount = u64::from_le_bytes(swap_state_account.data[32..40].try_into()?);
    let output_mint_bytes: [u8; 32] = swap_state_account.data[40..72].try_into()?;
    let output_mint = Pubkey::new_from_array(output_mint_bytes);
    let recipient_ata_bytes: [u8; 32] = swap_state_account.data[72..104].try_into()?;
    let recipient_ata = Pubkey::new_from_array(recipient_ata_bytes);
    let min_output_amount = u64::from_le_bytes(swap_state_account.data[104..112].try_into()?);

    println!("📊 Swap Details:");
    println!(
        "   SOL Amount: {} ({} SOL)",
        sol_amount,
        sol_amount as f64 / 1e9
    );
    println!("   Output Mint: {}", output_mint);
    println!("   Recipient ATA: {}", recipient_ata);
    println!("   Min Output Amount: {} tokens", min_output_amount);
    println!();

    // Step 1: Execute Jupiter Swap (SKIPPED - requires Jupiter integration)
    println!("🔄 Step 1: Executing Jupiter swap...");
    println!("   ⚠️  Automated Jupiter swap not implemented in this tool");
    println!("   You need to manually:");
    println!(
        "   1. Use the relay's SOL to swap {} SOL → {} on Jupiter",
        sol_amount as f64 / 1e9,
        output_mint
    );
    println!("   2. Send the output tokens to: {}", recipient_ata);
    println!(
        "   3. Ensure at least {} tokens are received",
        min_output_amount
    );
    println!();
    println!("   Once the swap is done, press ENTER to continue with ExecuteSwap...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!();

    // Step 2: Execute ExecuteSwap instruction
    println!("🔄 Step 2: Calling ExecuteSwap to close PDA...");

    let token_program = spl_token::id();

    // Build ExecuteSwap instruction
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(swap_state_pda, false),
            AccountMeta::new_readonly(recipient_ata, false),
            AccountMeta::new(relay_pubkey, true),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: {
            let mut data = vec![2u8]; // ExecuteSwap discriminator
            data.extend_from_slice(&nullifier);
            data
        },
    };

    let recent_blockhash = client.get_latest_blockhash()?;
    let message = Message::new(&[ix], Some(&relay_pubkey));
    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = recent_blockhash;
    tx.sign(&[&relay_keypair], recent_blockhash);

    println!("   Sending ExecuteSwap transaction...");
    let exec_sig = client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow::anyhow!("Failed to execute ExecuteSwap: {}", e))?;

    println!("   ✅ ExecuteSwap completed: {}", exec_sig);
    println!("   View: https://orb.helius.dev/tx/{}?cluster=devnet", exec_sig);
    println!();

    // Verify PDA is closed
    match client.get_account(&swap_state_pda) {
        Ok(_) => {
            println!("⚠️  SwapState PDA still exists - ExecuteSwap may have failed");
        }
        Err(_) => {
            println!("✅ SwapState PDA closed - swap completed successfully!");
            println!(
                "   Relay reimbursed: {} lamports",
                swap_state_account.lamports
            );
        }
    }

    println!();
    println!("🎉 Swap completion process finished!");

    Ok(())
}
