/**
 * Withdraw Example
 *
 * Demonstrates how to withdraw funds from a previously deposited note.
 * You can withdraw the full amount (minus fees) or a specific amount.
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

  // For this example, we'll create and deposit a note first
  // In practice, you'd load a previously saved note
  console.log(`\n📝 Creating and depositing a note first...`);
  const depositAmount = 10_000_000; // 0.01 SOL
  const note = client.generateNote(depositAmount);

  const depositResult = await client.deposit(connection, note, {
    onProgress: (status: string) => console.log(`   ${status}`),
  });

  console.log(`✅ Note deposited!`);
  console.log(`   Leaf index: ${depositResult.leafIndex}`);
  console.log(`   Transaction: ${depositResult.signature}`);

  // Now withdraw from this deposited note
  const depositedNote = depositResult.note;
  const fee = calculateFee(depositedNote.amount);
  const withdrawableAmount = depositedNote.amount - fee;

  console.log(`\n💸 Withdrawing from deposited note...`);
  console.log(`   Note amount: ${formatAmount(depositedNote.amount)} SOL`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
  console.log(`   Withdrawable: ${formatAmount(withdrawableAmount)} SOL`);

  // Recipient address
  const recipient = Keypair.generate().publicKey;
  console.log(`   Recipient: ${recipient.toBase58()}`);

  // Withdraw full amount (minus fees)
  console.log(`\n🚀 Starting withdrawal (withdrawAll: true)...`);
  const withdrawResult = await client.withdraw(
    connection,
    depositedNote,
    recipient,
    {
      withdrawAll: true, // Withdraw full amount minus fees
      onProgress: (status: string) => console.log(`   ${status}`),
    }
  );

  console.log(`\n✅ Withdrawal successful!`);
  console.log(`   Transaction: ${withdrawResult.signature}`);
  console.log(`   Nullifier: ${withdrawResult.nullifier.slice(0, 16)}...`);
  console.log(`   Amount sent: ${formatAmount(withdrawResult.outputs[0].amount)} SOL`);
  console.log(`   To: ${withdrawResult.outputs[0].recipient}`);

  console.log(`\n💡 Note: You can also withdraw a specific amount:`);
  console.log(`   await client.withdraw(connection, note, recipient, {`);
  console.log(`     withdrawAll: false,`);
  console.log(`     amount: 5_000_000, // Specific amount in lamports`);
  console.log(`   });`);
}

main().catch((error) => {
  console.error("❌ Error:", error?.message ?? error);
  process.exit(1);
});

