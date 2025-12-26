/// Test Jupiter swap integration
///
/// This example tests the Jupiter client by:
/// 1. Getting a quote for SOL → USDC swap
/// 2. Printing the expected output and price impact
///
/// Run with: cargo run --package relay --example test_jupiter_swap
use relay::swap::jupiter::JupiterClient;
use relay::swap::types::QuoteRequest;

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("===========================================");
    println!("  Jupiter Swap Integration Test");
    println!("===========================================\n");

    // Create Jupiter client
    let client = JupiterClient::new();
    println!("✓ Jupiter client created\n");

    // Test 1: Get quote for 1 SOL → USDC
    println!("Test 1: Getting quote for 1 SOL → USDC");
    println!("---------------------------------------");

    let quote_request = QuoteRequest {
        input_mint: SOL_MINT.to_string(),
        output_mint: USDC_MINT.to_string(),
        amount: 1_000_000_000, // 1 SOL (9 decimals)
        slippage_bps: 50,      // 0.5% slippage
    };

    match client.get_quote(&quote_request).await {
        Ok(quote) => {
            println!("✓ Quote received successfully!\n");
            println!("Quote Details:");
            println!(
                "  Input: {} lamports ({} SOL)",
                quote.in_amount,
                quote.in_amount.parse::<u64>().unwrap_or(0) as f64 / 1e9
            );
            println!(
                "  Output: {} (≈ ${:.2})",
                quote.out_amount,
                quote.out_amount.parse::<u64>().unwrap_or(0) as f64 / 1e6
            );
            println!(
                "  Min output (with slippage): {}",
                quote.other_amount_threshold
            );
            println!("  Price impact: {}%", quote.price_impact_pct);
            println!("  Input mint: {}", quote.input_mint);
            println!("  Output mint: {}", quote.output_mint);
        }
        Err(e) => {
            println!("✗ Quote failed: {}", e);
            println!("\nNote: This test requires internet access to reach Jupiter API");
            return Err(e.into());
        }
    }

    println!("\n===========================================");
    println!("  All Tests Passed!");
    println!("===========================================");

    Ok(())
}
