/**
 * Simple Deposit Example
 * 
 * Demonstrates a single deposit operation.
 * Safe to run multiple times - each deposit gets a unique leaf index.
 */

import { CloakSDK, formatAmount, calculateFee } from "@cloak/sdk";
import { Connection, Keypair } from "@solana/web3.js";
import { readFileSync } from "fs";
import * as path from "path";

async function main() {
  console.log("🎯 Simple Cloak Deposit Example\n");

  // Load keypair
  const homeDir = process.env.HOME || process.env.USERPROFILE;
  if (!homeDir) throw new Error("Could not determine home directory");

  const systemKeypairPath = path.join(homeDir, ".config", "solana", "id.json");
  const secretKeyString = readFileSync(systemKeypairPath, "utf8");
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  const keypair = Keypair.fromSecretKey(secretKey);
  
  console.log(`✅ Using keypair: ${keypair.publicKey.toBase58()}\n`);

  // Initialize SDK
  const client = new CloakSDK({
    network: "testnet",
    keypairBytes: keypair.secretKey,
  });

  // Setup connection
  const connection = new Connection("https://api.testnet.solana.com", "confirmed");

  // Calculate amounts
  const depositAmount = 100_000_000; // 0.1 SOL
  const fee = calculateFee(depositAmount);

  console.log(`📥 Depositing ${formatAmount(depositAmount)} SOL`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
  console.log(`   Net amount: ${formatAmount(depositAmount - fee)} SOL\n`);

  try {
    const result = await client.deposit(
      connection,
      depositAmount,
      {
        onProgress: (status: string) => {
          console.log(`   ${status}`);
        },
      }
    );

    console.log(`\n✅ Deposit successful!`);
    console.log(`   Transaction: ${result.signature}`);
    console.log(`   Leaf index: ${result.leafIndex}`);
    console.log(`   Root: ${result.root.slice(0, 16)}...`);

    // Save the note
    const noteJson = client.exportNote(result.note, true);
    console.log(`\n💾 Your note (save this securely!):`);
    console.log(noteJson);

    // Verify note is withdrawable
    const isWithdrawable = client.isWithdrawable(result.note);
    console.log(`\n✅ Note is withdrawable: ${isWithdrawable}`);

    console.log(`\n🎉 Success! You can now:`);
    console.log(`   1. Save the note to a file`);
    console.log(`   2. Use it for withdrawal later`);
    console.log(`   3. Share it privately (treat like cash!)`);

  } catch (error: any) {
    console.error(`\n❌ Deposit failed:`, error.message);
    
    if (error.message.includes("duplicate key")) {
      console.log(`\n💡 This might be a race condition.`);
      console.log(`   Try running the deposit again.`);
    } else if (error.message.includes("insufficient funds")) {
      console.log(`\n💡 You need at least ${formatAmount(depositAmount + 5000)} SOL`);
      console.log(`   (${formatAmount(depositAmount)} deposit + transaction fees)`);
    }
    
    throw error;
  }
}

main().catch((error) => {
  console.error("Fatal error:", error?.message ?? error);
  process.exit(1);
});

