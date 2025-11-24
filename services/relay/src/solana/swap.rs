use crate::solana::{Error, SolanaClient};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::str::FromStr;
use tracing::{error, info, warn};

const JUPITER_QUOTE_API_V1: &str = "https://lite-api.jup.ag/swap/v1/quote";
const JUPITER_SWAP_API_V1: &str = "https://lite-api.jup.ag/swap/v1/swap";

// Token mints
const WRAPPED_SOL: &str = "So11111111111111111111111111111111111111112";
const DEVNET_USDC: &str = "BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k"; // DevUSDC (has good Orca pool)

#[derive(Debug, Serialize)]
struct JupiterQuoteRequest {
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    amount: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: u16,
    #[serde(rename = "restrictIntermediateTokens")]
    restrict_intermediate_tokens: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct JupiterQuoteResponse {
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "inAmount")]
    in_amount: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    #[serde(rename = "outAmount")]
    out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    swap_mode: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: u16,
    #[serde(rename = "priceImpactPct")]
    price_impact_pct: String,
    #[serde(rename = "routePlan")]
    route_plan: Vec<RoutePlanStep>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RoutePlanStep {
    #[serde(rename = "swapInfo")]
    swap_info: SwapInfo,
    percent: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct SwapInfo {
    #[serde(rename = "ammKey")]
    amm_key: String,
    label: Option<String>,
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    #[serde(rename = "inAmount")]
    in_amount: String,
    #[serde(rename = "outAmount")]
    out_amount: String,
    #[serde(rename = "feeAmount")]
    fee_amount: String,
    #[serde(rename = "feeMint")]
    fee_mint: String,
}

#[derive(Debug, Serialize)]
struct JupiterSwapRequest {
    #[serde(rename = "quoteResponse")]
    quote_response: JupiterQuoteResponse,
    #[serde(rename = "userPublicKey")]
    user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    wrap_and_unwrap_sol: bool,
    #[serde(rename = "dynamicComputeUnitLimit")]
    dynamic_compute_unit_limit: bool,
    #[serde(rename = "prioritizationFeeLamports")]
    prioritization_fee_lamports: String,
}

#[derive(Debug, Deserialize)]
struct JupiterSwapResponse {
    #[serde(rename = "swapTransaction")]
    swap_transaction: String,
    #[serde(rename = "lastValidBlockHeight")]
    last_valid_block_height: u64,
}

/// Perform a token swap using Jupiter or Orca
///
/// # Arguments
/// * `client` - Solana client trait object
/// * `relay_keypair` - The relay's keypair (has the SOL to swap)
/// * `input_amount_lamports` - Amount of SOL to swap (in lamports)
/// * `output_mint` - The token mint to swap to
/// * `min_output_amount` - Minimum output amount expected
/// * `recipient_ata` - The recipient's associated token account for output tokens
pub async fn perform_swap(
    client: &dyn SolanaClient,
    relay_keypair: &Keypair,
    input_amount_lamports: u64,
    output_mint: Pubkey,
    min_output_amount: u64,
    recipient_ata: Pubkey,
) -> Result<String, Error> {
    info!(
        "🔄 Starting token swap: {} SOL → {} (min: {})",
        input_amount_lamports as f64 / 1e9,
        output_mint,
        min_output_amount
    );

    // Try Jupiter first
    match perform_jupiter_swap(
        client,
        relay_keypair,
        input_amount_lamports,
        output_mint,
        min_output_amount,
        recipient_ata,
    )
    .await
    {
        Ok(signature) => {
            info!("✅ Jupiter swap successful: {}", signature);
            return Ok(signature);
        }
        Err(e) => {
            warn!("⚠️ Jupiter swap failed: {}. Trying Orca...", e);
        }
    }

    // Fall back to Orca (not yet implemented)
    match perform_orca_swap(
        client,
        relay_keypair,
        input_amount_lamports,
        output_mint,
        min_output_amount,
        recipient_ata,
    )
    .await
    {
        Ok(signature) => {
            info!("✅ Orca swap successful: {}", signature);
            return Ok(signature);
        }
        Err(e) => {
            warn!("⚠️ Orca swap failed: {}", e);
        }
    }

    error!("❌ Swap failed");
    Err(Error::InternalServerError("Swap failed".to_string()))
}

async fn perform_jupiter_swap(
    client: &dyn SolanaClient,
    relay_keypair: &Keypair,
    input_amount_lamports: u64,
    output_mint: Pubkey,
    min_output_amount: u64,
    recipient_ata: Pubkey,
) -> Result<String, Error> {
    info!("Attempting Jupiter swap...");

    let http_client = reqwest::Client::new();

    // Step 1: Get quote
    let quote_params = JupiterQuoteRequest {
        input_mint: WRAPPED_SOL.to_string(),
        output_mint: output_mint.to_string(),
        amount: input_amount_lamports.to_string(),
        slippage_bps: 100, // 1% slippage
        restrict_intermediate_tokens: true,
    };

    let quote_url = format!(
        "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}&restrictIntermediateTokens=true",
        JUPITER_QUOTE_API_V1,
        quote_params.input_mint,
        quote_params.output_mint,
        quote_params.amount,
        quote_params.slippage_bps
    );

    let quote_response = http_client
        .get(&quote_url)
        .send()
        .await
        .map_err(|e| Error::NetworkError(format!("Failed to send quote request: {}", e)))?
        .json::<JupiterQuoteResponse>()
        .await
        .map_err(|e| Error::NetworkError(format!("Failed to parse Jupiter quote: {}", e)))?;

    info!(
        "  Quote: {} SOL → {} tokens",
        quote_response.in_amount.parse::<u64>().unwrap_or(0) as f64 / 1e9,
        quote_response.out_amount.parse::<u64>().unwrap_or(0) as f64 / 1e6 // Assuming 6 decimals
    );

    // Check if output meets minimum requirement
    let out_amount = quote_response
        .out_amount
        .parse::<u64>()
        .map_err(|e| Error::ValidationError(format!("Failed to parse output amount: {}", e)))?;

    if out_amount < min_output_amount {
        return Err(Error::ValidationError(format!(
            "Output amount {} is less than minimum required {}",
            out_amount, min_output_amount
        )));
    }

    // Step 2: Get swap transaction
    // Note: We use relay's keypair but specify recipient_ata for output
    // This requires creating an intermediate transaction to transfer tokens
    let swap_request = JupiterSwapRequest {
        quote_response,
        user_public_key: relay_keypair.pubkey().to_string(),
        wrap_and_unwrap_sol: true,
        dynamic_compute_unit_limit: true,
        prioritization_fee_lamports: "auto".to_string(),
    };

    let swap_response = http_client
        .post(JUPITER_SWAP_API_V1)
        .json(&swap_request)
        .send()
        .await
        .map_err(|e| Error::NetworkError(format!("Failed to send swap request: {}", e)))?
        .json::<JupiterSwapResponse>()
        .await
        .map_err(|e| {
            Error::NetworkError(format!("Failed to parse Jupiter swap response: {}", e))
        })?;

    // Step 3: Sign and send the swap transaction
    let tx_bytes = BASE64
        .decode(&swap_response.swap_transaction)
        .map_err(|e| Error::SerializationError(format!("Failed to decode transaction: {}", e)))?;

    let mut transaction: Transaction = bincode::deserialize(&tx_bytes).map_err(|e| {
        Error::SerializationError(format!("Failed to deserialize transaction: {}", e))
    })?;

    let recent_blockhash = client.get_latest_blockhash().await?;
    transaction.sign(&[relay_keypair], recent_blockhash);

    let signature = client.send_and_confirm_transaction(&transaction).await?;

    info!("  Swap transaction confirmed: {}", signature);

    // Step 4: Transfer output tokens to recipient
    // The swap sends tokens to relay's ATA, we need to transfer to recipient's ATA
    let relay_ata = get_associated_token_address(&relay_keypair.pubkey(), &output_mint);

    // Fetch the actual balance received
    let relay_ata_account = client.get_account(&relay_ata).await?;

    // Parse token account data to get balance
    // Token account data layout: [mint(32), owner(32), amount(8), ...]
    if relay_ata_account.data.len() < 72 {
        return Err(Error::InternalServerError(
            "Invalid token account data".to_string(),
        ));
    }
    let amount_bytes: [u8; 8] = relay_ata_account.data[64..72]
        .try_into()
        .map_err(|_| Error::InternalServerError("Failed to parse amount".to_string()))?;
    let actual_output_amount = u64::from_le_bytes(amount_bytes);

    info!(
        "  Received {} tokens from Jupiter swap (expected: {})",
        actual_output_amount, out_amount
    );

    // Transfer ALL tokens to recipient
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        &relay_ata,
        &recipient_ata,
        &relay_keypair.pubkey(),
        &[&relay_keypair.pubkey()],
        actual_output_amount, // Transfer exact amount received
    )
    .map_err(|e| {
        Error::InternalServerError(format!("Failed to create transfer instruction: {}", e))
    })?;

    let recent_blockhash2 = client.get_latest_blockhash().await?;
    let mut transfer_tx =
        Transaction::new_with_payer(&[transfer_ix], Some(&relay_keypair.pubkey()));
    transfer_tx.sign(&[relay_keypair], recent_blockhash2);

    let transfer_sig = client.send_and_confirm_transaction(&transfer_tx).await?;

    info!("  Token transfer confirmed: {}", transfer_sig);

    // Return the transfer signature since that's the transaction that actually changes
    // the recipient's USDC balance
    Ok(transfer_sig.to_string())
}

async fn perform_orca_swap(
    client: &dyn SolanaClient,
    relay_keypair: &Keypair,
    input_amount_lamports: u64,
    output_mint: Pubkey,
    min_output_amount: u64,
    recipient_ata: Pubkey,
) -> Result<String, Error> {
    info!("Attempting Orca swap...");

    use orca_whirlpools_client::{get_whirlpool_address, Whirlpool};
    use std::str::FromStr;

    // Orca Whirlpool config on devnet
    let whirlpool_config = Pubkey::from_str("FcrweFY1G9HJAHG5inkGB6pKg1HZ6x9UC2WioAfWrGkR")
        .map_err(|e| Error::InternalServerError(format!("Invalid whirlpool config: {}", e)))?;

    let wsol_mint = Pubkey::from_str(WRAPPED_SOL)
        .map_err(|e| Error::InternalServerError(format!("Invalid wSOL mint: {}", e)))?;

    // Try multiple tick spacings until we find an existing pool
    let tick_spacings = vec![64, 8, 128, 1];

    for tick_spacing in tick_spacings {
        info!("  Trying tick spacing {}...", tick_spacing);

        // Get whirlpool address for this tick spacing
        let (whirlpool_address, _) =
            get_whirlpool_address(&whirlpool_config, &wsol_mint, &output_mint, tick_spacing)
                .map_err(|e| {
                    warn!("  Failed to derive whirlpool address: {:?}", e);
                    Error::InternalServerError(format!(
                        "Failed to derive whirlpool address: {:?}",
                        e
                    ))
                })?;

        // Check if pool exists by fetching it
        let whirlpool_account = match client.get_account(&whirlpool_address).await {
            Ok(acc) => acc,
            Err(_) => {
                warn!("  Pool not found for tick spacing {}", tick_spacing);
                continue; // Try next tick spacing
            }
        };

        // Deserialize to get the actual tick_spacing from the pool
        let whirlpool_data = match Whirlpool::from_bytes(&whirlpool_account.data) {
            Ok(data) => data,
            Err(e) => {
                warn!("  Failed to parse whirlpool data: {:?}", e);
                continue;
            }
        };

        // Use the pool's actual tick spacing (not our guess)
        let actual_tick_spacing = whirlpool_data.tick_spacing;
        info!(
            "  Found pool with actual tick spacing: {}",
            actual_tick_spacing
        );

        match perform_orca_swap_with_pool(
            client,
            relay_keypair,
            input_amount_lamports,
            &wsol_mint,
            &output_mint,
            min_output_amount,
            recipient_ata,
            whirlpool_address,
            whirlpool_data,
        )
        .await
        {
            Ok(signature) => {
                info!("  ✅ Success with tick spacing {}", actual_tick_spacing);
                return Ok(signature);
            }
            Err(e) => {
                warn!("  ❌ Tick spacing {} failed: {}", actual_tick_spacing, e);
                continue;
            }
        }
    }

    Err(Error::InternalServerError(
        "All Orca pools failed. Devnet pools may be too imbalanced.".to_string(),
    ))
}

async fn perform_orca_swap_with_pool(
    client: &dyn SolanaClient,
    relay_keypair: &Keypair,
    input_amount_lamports: u64,
    wsol_mint: &Pubkey,
    output_mint: &Pubkey,
    min_output_amount: u64,
    recipient_ata: Pubkey,
    whirlpool_address: Pubkey,
    whirlpool_data: orca_whirlpools_client::Whirlpool,
) -> Result<String, Error> {
    use orca_whirlpools_client::{get_oracle_address, get_tick_array_address, SwapV2Builder};

    // Create relay's wSOL ATA
    let relay_wsol_ata = get_associated_token_address(&relay_keypair.pubkey(), wsol_mint);

    // Ensure relay has wSOL ATA
    ensure_ata_exists(client, &relay_keypair.pubkey(), wsol_mint, relay_keypair).await?;

    // Wrap SOL to wSOL
    wrap_sol_to_wsol(client, relay_keypair, input_amount_lamports).await?;

    // Determine swap direction
    let a_to_b = whirlpool_data.token_mint_a.to_bytes() == wsol_mint.to_bytes();

    // Calculate swap limit
    let sqrt_price_limit = if a_to_b { 4295048016 } else { u128::MAX }; // Min/max sqrt price

    info!(
        "  Swapping {} lamports, direction: a_to_b={}",
        input_amount_lamports, a_to_b
    );

    // Get oracle address
    let (oracle_address, _) = get_oracle_address(&whirlpool_address)
        .map_err(|e| Error::InternalServerError(format!("Failed to derive oracle: {:?}", e)))?;

    // Get tick_spacing from the actual pool
    let tick_spacing = whirlpool_data.tick_spacing;
    let tick_current = whirlpool_data.tick_current_index;

    // Use orca_whirlpools_core to calculate the correct tick array start indices
    use orca_whirlpools_core::get_tick_array_start_tick_index;

    const TICK_ARRAY_SIZE: i32 = 88;
    let tick_array_spacing = TICK_ARRAY_SIZE * (tick_spacing as i32);

    // Get start tick index for the current tick array (containing tick_current)
    let start_tick_index_0 = get_tick_array_start_tick_index(tick_current, tick_spacing);

    // For swaps, we need 3 sequential tick arrays based on direction
    // If a_to_b (swap A for B, price going down), we traverse arrays in descending order
    // If b_to_a (swap B for A, price going up), we traverse arrays in ascending order
    let start_tick_index_1 = if a_to_b {
        start_tick_index_0 - tick_array_spacing
    } else {
        start_tick_index_0 + tick_array_spacing
    };

    let start_tick_index_2 = if a_to_b {
        start_tick_index_0 - tick_array_spacing * 2
    } else {
        start_tick_index_0 + tick_array_spacing * 2
    };

    info!(
        "  Tick current: {}, spacing: {}, a_to_b: {}",
        tick_current, tick_spacing, a_to_b
    );
    info!(
        "  Tick array indices: [{}, {}, {}]",
        start_tick_index_0, start_tick_index_1, start_tick_index_2
    );

    // Get tick array addresses
    let (tick_array_0, _) = get_tick_array_address(&whirlpool_address, start_tick_index_0)
        .map_err(|e| {
            Error::InternalServerError(format!("Failed to derive tick array 0: {:?}", e))
        })?;

    let (tick_array_1, _) = get_tick_array_address(&whirlpool_address, start_tick_index_1)
        .map_err(|e| {
            Error::InternalServerError(format!("Failed to derive tick array 1: {:?}", e))
        })?;

    let (tick_array_2, _) = get_tick_array_address(&whirlpool_address, start_tick_index_2)
        .map_err(|e| {
            Error::InternalServerError(format!("Failed to derive tick array 2: {:?}", e))
        })?;

    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
        .map_err(|e| Error::InternalServerError(format!("Invalid token program: {}", e)))?;

    let relay_output_ata = get_associated_token_address(&relay_keypair.pubkey(), output_mint);

    // Build swap instruction
    // Use a permissive threshold (1) since ExecuteSwap will verify the actual minimum
    // Orca pricing might differ from Jupiter quotes, especially on devnet
    let permissive_threshold = 1u64; // Accept any reasonable output

    let swap_ix = SwapV2Builder::new()
        .whirlpool(whirlpool_address)
        .token_program_a(token_program)
        .token_program_b(token_program)
        .memo_program(spl_memo::ID)
        .token_authority(relay_keypair.pubkey())
        .token_mint_a(Pubkey::new_from_array(
            whirlpool_data.token_mint_a.to_bytes(),
        ))
        .token_mint_b(Pubkey::new_from_array(
            whirlpool_data.token_mint_b.to_bytes(),
        ))
        .token_owner_account_a(if a_to_b {
            relay_wsol_ata
        } else {
            relay_output_ata
        })
        .token_vault_a(Pubkey::new_from_array(
            whirlpool_data.token_vault_a.to_bytes(),
        ))
        .token_owner_account_b(if a_to_b {
            relay_output_ata
        } else {
            relay_wsol_ata
        })
        .token_vault_b(Pubkey::new_from_array(
            whirlpool_data.token_vault_b.to_bytes(),
        ))
        .tick_array0(tick_array_0)
        .tick_array1(tick_array_1)
        .tick_array2(tick_array_2)
        .oracle(oracle_address)
        .amount(input_amount_lamports)
        .other_amount_threshold(permissive_threshold) // Permissive - ExecuteSwap will verify minimum
        .sqrt_price_limit(sqrt_price_limit)
        .amount_specified_is_input(true)
        .a_to_b(a_to_b)
        .instruction();

    // Build and sign transaction
    let recent_blockhash = client.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(&[swap_ix], Some(&relay_keypair.pubkey()));
    transaction.sign(&[relay_keypair], recent_blockhash);

    // Send transaction
    let signature = client.send_and_confirm_transaction(&transaction).await?;

    info!("  Orca swap confirmed: {}", signature);

    // Fetch the relay's output ATA to get the actual balance
    let relay_ata_account = client.get_account(&relay_output_ata).await?;

    // Parse token account data to get balance
    // Token account data layout: [mint(32), owner(32), amount(8), ...]
    if relay_ata_account.data.len() < 72 {
        return Err(Error::InternalServerError(
            "Invalid token account data".to_string(),
        ));
    }
    let amount_bytes: [u8; 8] = relay_ata_account.data[64..72]
        .try_into()
        .map_err(|_| Error::InternalServerError("Failed to parse amount".to_string()))?;
    let actual_output_amount = u64::from_le_bytes(amount_bytes);

    info!(
        "  Received {} tokens from Orca swap (min required: {})",
        actual_output_amount, min_output_amount
    );

    // Transfer ALL tokens from relay's output ATA to recipient's ATA
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        &relay_output_ata,
        &recipient_ata,
        &relay_keypair.pubkey(),
        &[&relay_keypair.pubkey()],
        actual_output_amount, // Transfer all tokens received
    )
    .map_err(|e| {
        Error::InternalServerError(format!("Failed to create transfer instruction: {}", e))
    })?;

    let recent_blockhash2 = client.get_latest_blockhash().await?;
    let mut transfer_tx =
        Transaction::new_with_payer(&[transfer_ix], Some(&relay_keypair.pubkey()));
    transfer_tx.sign(&[relay_keypair], recent_blockhash2);

    let transfer_sig = client.send_and_confirm_transaction(&transfer_tx).await?;

    info!("  Token transfer to recipient confirmed: {}", transfer_sig);

    // Return the transfer signature since that's the transaction that actually changes
    // the recipient's USDC balance
    Ok(transfer_sig.to_string())
}

/// Wrap SOL to wSOL in the relay's ATA
async fn wrap_sol_to_wsol(
    client: &dyn SolanaClient,
    relay_keypair: &Keypair,
    amount_lamports: u64,
) -> Result<(), Error> {
    let wsol_mint = Pubkey::from_str(WRAPPED_SOL)
        .map_err(|e| Error::InternalServerError(format!("Invalid wSOL mint: {}", e)))?;

    let relay_wsol_ata = get_associated_token_address(&relay_keypair.pubkey(), &wsol_mint);

    // Create wSOL ATA if needed
    ensure_ata_exists(client, &relay_keypair.pubkey(), &wsol_mint, relay_keypair).await?;

    // Transfer SOL to wSOL ATA and sync native
    let transfer_ix = solana_sdk::system_instruction::transfer(
        &relay_keypair.pubkey(),
        &relay_wsol_ata,
        amount_lamports,
    );

    let sync_native_ix = spl_token::instruction::sync_native(&spl_token::id(), &relay_wsol_ata)
        .map_err(|e| {
            Error::InternalServerError(format!("Failed to create sync_native instruction: {}", e))
        })?;

    let recent_blockhash = client.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(
        &[transfer_ix, sync_native_ix],
        Some(&relay_keypair.pubkey()),
    );
    transaction.sign(&[relay_keypair], recent_blockhash);

    client.send_and_confirm_transaction(&transaction).await?;

    info!("  Wrapped {} lamports to wSOL", amount_lamports);

    Ok(())
}

/// Create associated token account if it doesn't exist
pub async fn ensure_ata_exists(
    client: &dyn SolanaClient,
    owner: &Pubkey,
    mint: &Pubkey,
    payer_keypair: &Keypair,
) -> Result<Pubkey, Error> {
    let ata = get_associated_token_address(owner, mint);

    // Check if ATA exists
    match client.get_account(&ata).await {
        Ok(_) => {
            info!("ATA already exists: {}", ata);
            Ok(ata)
        }
        Err(_) => {
            info!("Creating ATA: {}", ata);

            let create_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &payer_keypair.pubkey(),
                    owner,
                    mint,
                    &spl_token::id(),
                );

            let recent_blockhash = client.get_latest_blockhash().await?;
            let mut tx =
                Transaction::new_with_payer(&[create_ata_ix], Some(&payer_keypair.pubkey()));
            tx.sign(&[payer_keypair], recent_blockhash);

            client.send_and_confirm_transaction(&tx).await?;

            Ok(ata)
        }
    }
}
