/**
 * Swap Example
 *
 * Demonstrates how to swap SOL for tokens privately using the swap() method.
 * 
 * Requirements:
 * - @solana/spl-token package installed
 * - Valid token mint address on devnet
 * 
 * The example uses Orca's swap quote API to get real-time swap quotes.
 * Make sure the output token mint exists on devnet.
 */

import { CloakSDK, formatAmount, calculateFee } from "@cloak/sdk";
import { Connection, Keypair } from "@solana/web3.js";
import { readFileSync } from "fs";
import * as path from "path";

async function main() {
  // Initialize connection and keypair
  const homeDir = process.env.HOME || process.env.USERPROFILE;
  if (!homeDir) throw new Error("Could not determine the user's home directory for keypair loading.");

  const systemKeypairPath = path.join(homeDir, ".config", "solana", "id.json");
  const secretKeyString = readFileSync(systemKeypairPath, "utf8");
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  const keypair = Keypair.fromSecretKey(secretKey);
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");

  // Initialize SDK
  const client = new CloakSDK({
    network: "devnet",
    keypairBytes: keypair.secretKey,
  });

  console.log("✅ Cloak client initialized");
  console.log(`   Using keypair: ${keypair.publicKey.toBase58()}`);

  // Note: @solana/spl-token is required for swap functionality
  // The SDK will attempt to import it dynamically when needed

  // Amount to swap
  const swapAmount = 10_000_000; // 0.01 SOL in lamports
  const fee = calculateFee(swapAmount);
  const swapableAmount = swapAmount - fee;

  console.log(`\n🔄 Private swap: SOL → Token`);
  console.log(`   Input amount: ${formatAmount(swapAmount)} SOL`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
  console.log(`   Amount to swap: ${formatAmount(swapableAmount)} SOL`);

  // Generate a new note
  const note = client.generateNote(swapAmount);
  console.log(`\n📝 Generated note (will be deposited automatically)`);

  // Output token configuration
  // Using a valid devnet token (you can change this to any token mint on devnet)
  // Example: BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k (from Orca devnet pools)
  const outputMint = "BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k";
  const slippageBps = 100; // 1% slippage tolerance

  console.log(`   Output token: ${outputMint}`);
  console.log(`   Slippage: ${slippageBps} bps (${slippageBps / 100}%)`);

  // Recipient (will receive tokens)
  const recipient = Keypair.generate().publicKey;
  console.log(`   Recipient: ${recipient.toBase58()}`);

  // Quote function - uses Orca's swap quote API
  const getQuote = async (
    amountLamports: number,
    mint: string,
    slippage: number
  ): Promise<{ outAmount: number; minOutputAmount: number }> => {
    console.log(`\n📊 Fetching swap quote from Orca...`);
    console.log(`   Amount: ${formatAmount(amountLamports)} SOL`);
    console.log(`   Output mint: ${mint}`);
    console.log(`   Slippage: ${slippage} bps`);

    // Use Orca's swap quote API
    // Native SOL mint: So11111111111111111111111111111111111111112
    const SOL_MINT = "So11111111111111111111111111111111111111112";
    const url = new URL("https://pools-api.devnet.orca.so/swap-quote");
    url.searchParams.set("from", SOL_MINT);
    url.searchParams.set("to", mint);
    url.searchParams.set("amount", amountLamports.toString());
    url.searchParams.set("isLegacy", "false");
    url.searchParams.set("amountIsInput", "true");
    url.searchParams.set("includeData", "true");
    url.searchParams.set("includeComputeBudget", "false");
    url.searchParams.set("maxTxSize", "1185");

    try {
      const response = await fetch(url.toString(), { method: "GET" });
      
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
          `Orca API returned error: ${response.status} - ${errorText.slice(0, 100)}`
        );
      }

      const json = (await response.json()) as {
        data?: {
          swap?: {
            outputAmount?: string;
          };
        };
        error?: string;
      };

      if (json.error) {
        throw new Error(`Orca API error: ${json.error}`);
      }

      if (!json.data?.swap?.outputAmount) {
        throw new Error("Orca API returned invalid response format");
      }

      const estimatedOut = parseInt(json.data.swap.outputAmount, 10);
      // Calculate min output with slippage tolerance
      const minOutputAmount = Math.floor(
        (estimatedOut * (10000 - slippage)) / 10000
      );

      console.log(`   ✅ Quote received from Orca`);
      console.log(`   Estimated output: ${estimatedOut} tokens`);
      console.log(`   Min output (with ${slippage}bps slippage): ${minOutputAmount} tokens`);

      return {
        outAmount: estimatedOut,
        minOutputAmount: minOutputAmount,
      };
    } catch (error: any) {
      console.log(`   ⚠️  Failed to fetch quote: ${error.message}`);
      console.log(`   💡 Make sure you're using a valid token mint on devnet`);
      console.log(`   💡 Orca API: https://pools-api.devnet.orca.so/swap-quote`);
      throw error;
    }
  };

  // Perform swap
  console.log(`\n🚀 Starting swap...`);
  try {
    const swapResult = await client.swap(
      connection,
      note,
      recipient,
      {
        outputMint: outputMint,
        slippageBps: slippageBps,
        getQuote: getQuote,
        onProgress: (status: string) => console.log(`   ${status}`),
      }
    );

    console.log(`\n✅ Swap successful!`);
    console.log(`   Transaction: ${swapResult.signature}`);
    console.log(`   Output mint: ${swapResult.outputMint}`);
    console.log(`   Min output: ${swapResult.minOutputAmount} tokens`);
    console.log(`   Recipient: ${swapResult.outputs[0].recipient}`);
  } catch (error: any) {
    if (error.message.includes("@solana/spl-token")) {
      console.log(`\n❌ Swap failed: ${error.message}`);
      console.log(`\n💡 To fix:`);
      console.log(`   1. Install @solana/spl-token: npm install @solana/spl-token`);
      console.log(`   2. Ensure the output token mint exists on devnet`);
    } else if (error.message.includes("quote") || error.message.includes("fetch")) {
      console.log(`\n❌ Swap failed: ${error.message}`);
      console.log(`\n💡 To fix:`);
      console.log(`   1. Make sure the token mint exists on devnet`);
      console.log(`   2. Check Orca API is accessible: https://pools-api.devnet.orca.so/swap-quote`);
    } else {
      throw error;
    }
  }
}

main().catch((error) => {
  console.error("❌ Error:", error?.message ?? error);
  process.exit(1);
});

