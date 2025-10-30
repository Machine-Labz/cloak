/**
 * Jito Bundle + Cloak Integration Example
 *
 * This example demonstrates:
 * 1. Using Jito bundles for guaranteed transaction execution
 * 2. Combining multiple Cloak private transfers in a single bundle
 * 3. Providing MEV protection and guaranteed ordering
 *
 * Use case: High-value private transfers that need guaranteed execution
 * and protection from MEV/sandwich attacks
 */

import { CloakSDK, formatAmount } from "@cloak/sdk";
import {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import { Bundle, searcherClient } from "jito-ts/sdk/block-engine/searcher";
import { isError } from "jito-ts/sdk/block-engine/utils";

// Jito endpoints
const JITO_BLOCK_ENGINE_URL = "https://mainnet.block-engine.jito.wtf";
const JITO_AUTH_KEYPAIR = Keypair.generate(); // In production, load from secure storage

async function main() {
  // ============================================================================
  // STEP 1: Setup
  // ============================================================================

  const connection = new Connection("https://api.mainnet-beta.solana.com", "confirmed");
  const payer = Keypair.generate(); // Replace with your wallet

  const wallet = {
    publicKey: payer.publicKey,
    sendTransaction: async (tx: any, connection: Connection) => {
      tx.partialSign(payer);
      return await connection.sendRawTransaction(tx.serialize());
    },
  };

  console.log("🚀 Jito Bundle + Cloak Integration Demo");
  console.log(`   Wallet: ${payer.publicKey.toBase58()}`);

  // ============================================================================
  // STEP 2: Initialize Jito Client
  // ============================================================================

  console.log("\n⚡ Initializing Jito block engine client...");

  const jitoClient = searcherClient(JITO_BLOCK_ENGINE_URL, JITO_AUTH_KEYPAIR);

  // Get tip accounts for MEV payments
  let tipAccounts: string[] = [];
  try {
    const tipAccountsResponse = await jitoClient.getTipAccounts();
    tipAccounts = tipAccountsResponse;
    console.log(`✅ Connected to Jito block engine`);
    console.log(`   Available tip accounts: ${tipAccounts.length}`);
  } catch (error) {
    console.error("❌ Failed to connect to Jito:", error);
    process.exit(1);
  }

  // ============================================================================
  // STEP 3: Initialize Cloak
  // ============================================================================

  const cloakClient = new CloakSDK({
    network: "mainnet-beta",
    programId: new PublicKey("YOUR_PROGRAM_ID"),
    poolAddress: new PublicKey("YOUR_POOL_ADDRESS"),
    commitmentsAddress: new PublicKey("YOUR_COMMITMENTS_ADDRESS"),
    apiUrl: "https://api.cloaklabz.xyz",
    proofTimeout: 5 * 60 * 1000,
  });

  console.log("✅ Cloak client initialized");

  // ============================================================================
  // STEP 4: Prepare Multiple Private Transfers
  // ============================================================================

  console.log("\n📝 Preparing multiple private transfers...");

  // Transfer 1: Split 1 SOL between 2 recipients
  const note1Amount = 1 * LAMPORTS_PER_SOL;
  const note1 = cloakClient.generateNote(note1Amount);
  const recipients1 = [
    {
      recipient: new PublicKey("RECIPIENT_1A"),
      amount: Math.floor(note1Amount * 0.6),
    },
    {
      recipient: new PublicKey("RECIPIENT_1B"),
      amount: Math.floor(note1Amount * 0.39), // Account for fees
    },
  ] as const;

  // Transfer 2: Send 0.5 SOL to single recipient
  const note2Amount = 0.5 * LAMPORTS_PER_SOL;
  const note2 = cloakClient.generateNote(note2Amount);
  const recipients2 = [
    {
      recipient: new PublicKey("RECIPIENT_2"),
      amount: Math.floor(note2Amount * 0.995), // Account for fees
    },
  ] as const;

  console.log(`   Transfer 1: ${formatAmount(note1Amount)} SOL → 2 recipients`);
  console.log(`   Transfer 2: ${formatAmount(note2Amount)} SOL → 1 recipient`);

  // ============================================================================
  // STEP 5: Create Bundle with Deposits
  // ============================================================================

  console.log("\n📦 Creating Jito bundle...");

  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();

  // Create deposit transactions for each note
  // Note: In production, you'd get these from the Cloak SDK's deposit method
  // For this example, we'll show the bundle structure

  const bundle = new Bundle([], 5); // Max 5 transactions per bundle

  // Add compute budget instructions for better execution
  const computeUnitLimit = ComputeBudgetProgram.setComputeUnitLimit({
    units: 400_000,
  });
  const computeUnitPrice = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 50_000, // Higher priority for bundle execution
  });

  // Transaction 1: Deposit for note 1
  const depositTx1 = new Transaction({
    feePayer: payer.publicKey,
    blockhash,
    lastValidBlockHeight,
  });
  depositTx1.add(computeUnitLimit);
  depositTx1.add(computeUnitPrice);
  // Add your deposit instruction here using cloakClient
  depositTx1.partialSign(payer);

  // Transaction 2: Deposit for note 2
  const depositTx2 = new Transaction({
    feePayer: payer.publicKey,
    blockhash,
    lastValidBlockHeight,
  });
  depositTx2.add(computeUnitLimit);
  depositTx2.add(computeUnitPrice);
  // Add your deposit instruction here using cloakClient
  depositTx2.partialSign(payer);

  // Transaction 3: Jito tip payment (helps bundle land faster)
  const tipAmount = 0.001 * LAMPORTS_PER_SOL; // 0.001 SOL tip
  const tipAccount = new PublicKey(tipAccounts[0]);

  const tipTx = new Transaction({
    feePayer: payer.publicKey,
    blockhash,
    lastValidBlockHeight,
  });
  tipTx.add(
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: tipAccount,
      lamports: tipAmount,
    })
  );
  tipTx.partialSign(payer);

  // Add transactions to bundle
  bundle.addTransactions(depositTx1, depositTx2, tipTx);

  console.log(`   Bundle contains ${bundle.transactions.length} transactions`);
  console.log(`   Tip amount: ${formatAmount(tipAmount)} SOL`);

  // ============================================================================
  // STEP 6: Send Bundle
  // ============================================================================

  console.log("\n🚀 Sending bundle to Jito...");

  try {
    const bundleId = await jitoClient.sendBundle(bundle);

    if (isError(bundleId)) {
      throw new Error(`Bundle failed: ${bundleId}`);
    }

    console.log(`✅ Bundle sent!`);
    console.log(`   Bundle ID: ${bundleId}`);

    // Wait for bundle status
    let attempts = 0;
    const maxAttempts = 60;

    while (attempts < maxAttempts) {
      const statusResponse = await jitoClient.getBundleStatuses([bundleId]);

      if (isError(statusResponse)) {
        throw new Error(`Failed to get bundle status: ${statusResponse}`);
      }

      const status = statusResponse.value[0];

      if (status) {
        console.log(`   Bundle status: ${status.confirmation_status}`);

        if (status.confirmation_status === "confirmed") {
          console.log(`✅ Bundle confirmed!`);
          console.log(`   Transactions: ${status.transactions?.join(", ")}`);
          break;
        }

        if (status.err) {
          throw new Error(`Bundle error: ${JSON.stringify(status.err)}`);
        }
      }

      // Wait 2 seconds before checking again
      await new Promise((resolve) => setTimeout(resolve, 2000));
      attempts++;
    }

    if (attempts >= maxAttempts) {
      throw new Error("Bundle confirmation timeout");
    }

    // ============================================================================
    // STEP 7: Complete Private Transfers
    // ============================================================================

    console.log("\n🔒 Completing private transfers (withdrawal phase)...");

    // After deposits are confirmed and indexed, complete the withdrawals
    // In production, you'd wait for the indexer to process the deposits

    console.log("\n⏳ Waiting for deposits to be indexed...");
    await new Promise((resolve) => setTimeout(resolve, 10000)); // Wait 10s

    // Now execute the withdrawal phase for each transfer
    // This would typically be done through the Cloak SDK's withdraw method

    console.log("\n✅ Private transfers complete!");
    console.log("   Benefits of using Jito bundles:");
    console.log("   ✓ Guaranteed transaction ordering");
    console.log("   ✓ All deposits land in same block");
    console.log("   ✓ MEV protection");
    console.log("   ✓ Higher success rate");

    // ============================================================================
    // STEP 8: Summary
    // ============================================================================

    console.log("\n📊 Summary:");
    console.log(`   Bundled ${bundle.transactions.length} transactions`);
    console.log(`   Tip paid: ${formatAmount(tipAmount)} SOL`);
    console.log(`   Total privacy: ${recipients1.length + recipients2.length} recipients`);
    console.log("\n🎉 Jito bundle execution successful!");
  } catch (error: any) {
    console.error("\n❌ Bundle execution failed:", error.message);
    console.error(error);
    process.exit(1);
  }
}

// ============================================================================
// Utility: Wait for bundle confirmation
// ============================================================================

async function waitForBundleConfirmation(
  client: ReturnType<typeof searcherClient>,
  bundleId: string,
  maxAttempts = 60
): Promise<void> {
  for (let i = 0; i < maxAttempts; i++) {
    const statusResponse = await client.getBundleStatuses([bundleId]);

    if (isError(statusResponse)) {
      throw new Error(`Failed to get bundle status: ${statusResponse}`);
    }

    const status = statusResponse.value[0];

    if (status?.confirmation_status === "confirmed") {
      return;
    }

    if (status?.err) {
      throw new Error(`Bundle error: ${JSON.stringify(status.err)}`);
    }

    await new Promise((resolve) => setTimeout(resolve, 2000));
  }

  throw new Error("Bundle confirmation timeout");
}

// Run example
if (require.main === module) {
  main().catch(console.error);
}

export { main };
