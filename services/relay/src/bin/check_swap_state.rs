use std::{env, str::FromStr};

/// Quick utility to check SwapState PDA and help manually complete stuck swaps
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let nullifier_hex = if args.len() > 1 {
        args[1].clone()
    } else {
        // Default to our stuck swap
        "650dbce1b64abc09c9ede2421c1c9102ea161db012be12c61f7951b5bbed81c0".to_string()
    };

    let program_id_str = if args.len() > 2 {
        args[2].clone()
    } else {
        "c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp".to_string()
    };

    let rpc_url =
        env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    println!("🔍 Checking SwapState PDA");
    println!("   Nullifier: {}", nullifier_hex);
    println!("   Program ID: {}", program_id_str);
    println!("   RPC: {}", rpc_url);
    println!();

    // Decode nullifier
    let nullifier_bytes = hex::decode(&nullifier_hex)?;
    if nullifier_bytes.len() != 32 {
        anyhow::bail!("Nullifier must be 32 bytes");
    }
    let mut nullifier = [0u8; 32];
    nullifier.copy_from_slice(&nullifier_bytes);

    // Parse program ID
    let program_id = Pubkey::from_str(&program_id_str)?;

    // Derive SwapState PDA
    let (swap_state_pda, bump) =
        Pubkey::find_program_address(&[b"swap_state", &nullifier], &program_id);

    println!("📍 SwapState PDA: {}", swap_state_pda);
    println!("   Bump: {}", bump);
    println!();

    // Check if account exists
    let client = RpcClient::new(rpc_url);

    match client.get_account(&swap_state_pda) {
        Ok(account) => {
            println!("✅ SwapState PDA EXISTS!");
            println!("   Owner: {}", account.owner);
            println!(
                "   Lamports: {} ({} SOL)",
                account.lamports,
                account.lamports as f64 / 1e9
            );
            println!("   Data length: {} bytes", account.data.len());
            println!();

            if account.data.len() >= 121 {
                // Parse SwapState data
                // Layout: [nullifier(32)][sol_amount(8)][output_mint(32)][recipient_ata(32)]
                //         [min_output_amount(8)][created_slot(8)][bump(1)]

                let stored_nullifier = &account.data[0..32];
                let sol_amount = u64::from_le_bytes(account.data[32..40].try_into()?);
                let output_mint_bytes: [u8; 32] = account.data[40..72].try_into()?;
                let output_mint = Pubkey::new_from_array(output_mint_bytes);
                let recipient_ata_bytes: [u8; 32] = account.data[72..104].try_into()?;
                let recipient_ata = Pubkey::new_from_array(recipient_ata_bytes);
                let min_output_amount = u64::from_le_bytes(account.data[104..112].try_into()?);
                let created_slot = u64::from_le_bytes(account.data[112..120].try_into()?);
                let stored_bump = account.data[120];

                println!("📊 SwapState Contents:");
                println!("   Nullifier: {}", hex::encode(stored_nullifier));
                println!(
                    "   SOL Amount: {} ({} SOL)",
                    sol_amount,
                    sol_amount as f64 / 1e9
                );
                println!("   Output Mint: {}", output_mint);
                println!("   Recipient ATA: {}", recipient_ata);
                println!("   Min Output Amount: {}", min_output_amount);
                println!("   Created Slot: {}", created_slot);
                println!("   Bump: {}", stored_bump);
                println!();

                println!("💡 TO MANUALLY COMPLETE THIS SWAP:");
                println!(
                    "   1. Execute Jupiter swap: {} SOL → {} tokens (to {})",
                    sol_amount as f64 / 1e9,
                    output_mint,
                    recipient_ata
                );
                println!(
                    "   2. Call ExecuteSwap instruction with nullifier: {}",
                    nullifier_hex
                );
                println!(
                    "   3. This will close the PDA and return {} lamports to relay",
                    account.lamports
                );
            }
        }
        Err(e) => {
            println!("❌ SwapState PDA does NOT exist");
            println!("   Error: {}", e);
            println!();
            println!("💡 This means:");
            println!("   - The swap either completed successfully, or");
            println!("   - TX1 (WithdrawSwap) never executed");
        }
    }

    Ok(())
}
