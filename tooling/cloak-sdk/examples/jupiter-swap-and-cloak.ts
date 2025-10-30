/**
 * Jupiter + Cloak Integration Example
 *
 * This example demonstrates:
 * 1. Swapping USDC for SOL on Jupiter DEX aggregator
 * 2. Taking the swapped SOL and privately transferring it using Cloak
 * 3. Handling the complete flow with proper error handling
 *
 * Use case: Swap tokens and immediately send privately to recipients
 */

import { CloakSDK, formatAmount, calculateFee } from "@cloak/sdk";
import {
  Connection,
  PublicKey,
  Keypair,
  VersionedTransaction,
} from "@solana/web3.js";
import fetch from "cross-fetch";

// Jupiter Quote API response type
interface JupiterQuote {
  inputMint: string;
  inAmount: string;
  outputMint: string;
  outAmount: string;
  otherAmountThreshold: string;
  swapMode: string;
  slippageBps: number;
  priceImpactPct: string;
  routePlan: any[];
}

async function main() {
  // ============================================================================
  // STEP 1: Setup
  // ============================================================================

  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const payer = Keypair.generate(); // Replace with your wallet

  const wallet = {
    publicKey: payer.publicKey,
    sendTransaction: async (tx: any, connection: Connection) => {
      if (tx instanceof VersionedTransaction) {
        tx.sign([payer]);
        return await connection.sendRawTransaction(tx.serialize());
      }
      tx.partialSign(payer);
      return await connection.sendRawTransaction(tx.serialize());
    },
  };

  console.log("🚀 Jupiter + Cloak Integration Demo");
  console.log(`   Wallet: ${payer.publicKey.toBase58()}`);

  // ============================================================================
  // STEP 2: Configure Cloak
  // ============================================================================

  const cloakClient = new CloakSDK({
    network: "devnet",
    programId: new PublicKey("YOUR_PROGRAM_ID"),
    poolAddress: new PublicKey("YOUR_POOL_ADDRESS"),
    commitmentsAddress: new PublicKey("YOUR_COMMITMENTS_ADDRESS"),
    apiUrl: "https://api.cloaklabz.xyz",
    proofTimeout: 5 * 60 * 1000,
  });

  // ============================================================================
  // STEP 3: Get Jupiter Quote
  // ============================================================================

  const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // Mainnet USDC
  const SOL_MINT = "So11111111111111111111111111111111111111112"; // Wrapped SOL
  const inputAmount = 100_000_000; // 100 USDC (6 decimals)

  console.log("\n🔄 Step 1: Getting Jupiter quote...");

  try {
    const quoteResponse = await fetch(
      `https://quote-api.jup.ag/v6/quote?` +
        `inputMint=${USDC_MINT}` +
        `&outputMint=${SOL_MINT}` +
        `&amount=${inputAmount}` +
        `&slippageBps=50` // 0.5% slippage
    );

    if (!quoteResponse.ok) {
      throw new Error(`Jupiter quote failed: ${quoteResponse.statusText}`);
    }

    const quote: JupiterQuote = await quoteResponse.json();
    const expectedSolAmount = parseInt(quote.outAmount);

    console.log(`✅ Quote received:`);
    console.log(`   Input: ${inputAmount / 1_000_000} USDC`);
    console.log(`   Expected output: ${formatAmount(expectedSolAmount)} SOL`);
    console.log(`   Price impact: ${quote.priceImpactPct}%`);

    // ============================================================================
    // STEP 4: Execute Swap on Jupiter
    // ============================================================================

    console.log("\n💱 Step 2: Executing swap...");

    const swapResponse = await fetch("https://quote-api.jup.ag/v6/swap", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        quoteResponse: quote,
        userPublicKey: payer.publicKey.toString(),
        wrapAndUnwrapSol: true,
        // Use dynamic compute units and priority fee for better execution
        dynamicComputeUnitLimit: true,
        prioritizationFeeLamports: "auto",
      }),
    });

    if (!swapResponse.ok) {
      throw new Error(`Jupiter swap instruction failed: ${swapResponse.statusText}`);
    }

    const { swapTransaction } = await swapResponse.json();

    // Deserialize the transaction
    const swapTransactionBuf = Buffer.from(swapTransaction, "base64");
    const transaction = VersionedTransaction.deserialize(swapTransactionBuf);

    // Sign and send
    transaction.sign([payer]);
    const swapSignature = await connection.sendRawTransaction(
      transaction.serialize(),
      {
        skipPreflight: false,
        maxRetries: 3,
      }
    );

    console.log(`   Transaction sent: ${swapSignature}`);

    // Wait for confirmation
    const latestBlockhash = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature: swapSignature,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    console.log(`✅ Swap confirmed!`);
    console.log(`   Signature: ${swapSignature}`);

    // ============================================================================
    // STEP 5: Check SOL Balance
    // ============================================================================

    console.log("\n💰 Step 3: Checking SOL balance...");

    const balance = await connection.getBalance(payer.publicKey);
    console.log(`   Current balance: ${formatAmount(balance)} SOL`);

    if (balance < expectedSolAmount * 0.9) {
      // Allow 10% slippage
      throw new Error(
        `Insufficient balance after swap. Expected ~${formatAmount(expectedSolAmount)} SOL, got ${formatAmount(balance)} SOL`
      );
    }

    // ============================================================================
    // STEP 6: Private Transfer with Cloak
    // ============================================================================

    console.log("\n🔒 Step 4: Initiating private transfer...");

    // Use most of the balance, leaving some for transaction fees
    const transferAmount = Math.floor(balance * 0.95);
    const fee = calculateFee(transferAmount);
    const distributable = transferAmount - fee;

    console.log(`   Transfer amount: ${formatAmount(transferAmount)} SOL`);
    console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
    console.log(`   Distributable: ${formatAmount(distributable)} SOL`);

    // Define recipients (example: split between 3 addresses)
    const recipients = [
      {
        recipient: new PublicKey("RECIPIENT_ADDRESS_1"),
        amount: Math.floor(distributable * 0.5), // 50%
      },
      {
        recipient: new PublicKey("RECIPIENT_ADDRESS_2"),
        amount: Math.floor(distributable * 0.3), // 30%
      },
      {
        recipient: new PublicKey("RECIPIENT_ADDRESS_3"),
        amount: Math.floor(distributable * 0.2), // 20%
      },
    ];

    // Generate note (not deposited yet)
    const note = cloakClient.generateNote(transferAmount);

    // Execute private transfer (handles deposit + proof + withdraw)
    console.log("\n🔐 Executing private transfer...");

    const transferResult = await cloakClient.privateTransfer(
      connection,
      wallet,
      note,
      recipients,
      {
        onProgress: (status) => {
          console.log(`   ${status}`);
        },
      }
    );

    console.log("\n✅ Private transfer complete!");
    console.log(`   Withdrawal signature: ${transferResult.signature}`);
    console.log(`   Nullifier: ${transferResult.nullifier.slice(0, 16)}...`);
    console.log(`   Recipients received funds privately - no on-chain link!`);

    // ============================================================================
    // STEP 7: Summary
    // ============================================================================

    console.log("\n📊 Summary:");
    console.log(`   1. Swapped ${inputAmount / 1_000_000} USDC → ${formatAmount(expectedSolAmount)} SOL`);
    console.log(`   2. Privately distributed ${formatAmount(distributable)} SOL to 3 recipients`);
    console.log(`   3. Total privacy: Swap and recipients are not linked on-chain`);
    console.log("\n🎉 Transaction complete with full privacy!");
  } catch (error: any) {
    console.error("\n❌ Error:", error.message);
    console.error(error);
    process.exit(1);
  }
}

// Run example
if (require.main === module) {
  main().catch(console.error);
}

export { main };
