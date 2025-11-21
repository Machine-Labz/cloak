/// Example demonstrating how to use the swap-on-withdrawal feature
///
/// This example shows how to submit a withdraw request with a Jupiter swap.
///
/// Usage:
///   cargo run --example swap_withdraw_example
use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Cloak Swap-on-Withdrawal Example\n");

    // Example 1: Regular withdraw (backward compatible)
    println!("Example 1: Regular Withdraw (No Swap)");
    println!("=====================================");
    let regular_withdraw = json!({
        "outputs": [{
            "recipient": "9aQdpayZ7U9xmEjT2yv3pLb7JxJ9FqE3YC4xN8Z8QqVK",
            "amount": 1000000
        }],
        "policy": {
            "deposit_amount": 1000000,
            "fee": 5000
        },
        "public_inputs": {
            "root": "0x...",
            "nullifier": "0x...",
            "outputs_hash": "0x...",
            "amount": "0x..."
        },
        "proof_bytes": "base64_encoded_proof..."
    });
    println!("{}\n", serde_json::to_string_pretty(&regular_withdraw)?);

    // Example 2: Withdraw with swap to USDC
    println!("Example 2: Withdraw with Swap (SOL → USDC)");
    println!("==========================================");
    let swap_withdraw_usdc = json!({
        "outputs": [{
            "recipient": "9aQdpayZ7U9xmEjT2yv3pLb7JxJ9FqE3YC4xN8Z8QqVK",
            "amount": 1000000
        }],
        "swap": {
            "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",  // USDC
            "slippage_bps": 50  // 0.5% slippage
        },
        "policy": {
            "deposit_amount": 1000000,
            "fee": 5000
        },
        "public_inputs": {
            "root": "0x...",
            "nullifier": "0x...",
            "outputs_hash": "0x...",
            "amount": "0x..."
        },
        "proof_bytes": "base64_encoded_proof..."
    });
    println!("{}\n", serde_json::to_string_pretty(&swap_withdraw_usdc)?);

    // Example 3: Withdraw with swap to BONK (higher slippage)
    println!("Example 3: Withdraw with Swap (SOL → BONK)");
    println!("==========================================");
    let swap_withdraw_bonk = json!({
        "outputs": [{
            "recipient": "9aQdpayZ7U9xmEjT2yv3pLb7JxJ9FqE3YC4xN8Z8QqVK",
            "amount": 1000000
        }],
        "swap": {
            "output_mint": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",  // BONK
            "slippage_bps": 100  // 1% slippage (memecoins have higher volatility)
        },
        "policy": {
            "deposit_amount": 1000000,
            "fee": 5000
        },
        "public_inputs": {
            "root": "0x...",
            "nullifier": "0x...",
            "outputs_hash": "0x...",
            "amount": "0x..."
        },
        "proof_bytes": "base64_encoded_proof..."
    });
    println!("{}\n", serde_json::to_string_pretty(&swap_withdraw_bonk)?);

    // Example 4: Submit to relay (uncomment to actually test)
    /*
    println!("Example 4: Submitting to Relay");
    println!("==============================");

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3002/withdraw")
        .json(&swap_withdraw_usdc)
        .send()
        .await?;

    println!("Status: {}", response.status());
    let body = response.text().await?;
    println!("Response: {}\n", body);
    */

    // Example 5: Check swap status
    println!("Example 5: Checking Job Status");
    println!("===============================");
    println!("GET http://localhost:3002/status/<job_id>");
    println!("\nExpected response:");
    let status_response = json!({
        "status": "completed",
        "tx_signature": "5g8H...",
        "job_id": "uuid-here",
        "created_at": "2025-01-16T12:00:00Z",
        "completed_at": "2025-01-16T12:00:03Z"
    });
    println!("{}\n", serde_json::to_string_pretty(&status_response)?);

    println!("✅ Examples complete!");
    println!("\nCommon Token Mints:");
    println!("  USDC:  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    println!("  USDT:  Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
    println!("  BONK:  DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
    println!("  WIF:   EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm");
    println!("  SOL:   So11111111111111111111111111111111111111112\n");

    println!("Slippage Guidelines:");
    println!("  - Stablecoins:    10-50 bps (0.1-0.5%)");
    println!("  - Major tokens:   50-100 bps (0.5-1%)");
    println!("  - Memecoins:      100-300 bps (1-3%)");
    println!("  - Maximum:        1000 bps (10%)\n");

    Ok(())
}
