/**
 * Private Transfer Example
 *
 * Demonstrates how to privately send SOL to multiple recipients (1-5 recipients).
 * This uses privateTransfer() which handles the complete flow: deposit + withdraw.
 */

import { CloakSDK, formatAmount, calculateFee, generateNote } from "@cloaklabz/sdk";
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


  // Amount to transfer
  const transferAmount = 10_000_000; // 0.01 SOL in lamports
  const fee = calculateFee(transferAmount);
  const distributable = transferAmount - fee;

  console.log(`\n💸 Private transfer to multiple recipients`);
  console.log(`   Total amount: ${formatAmount(transferAmount)} SOL`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
  console.log(`   Distributable: ${formatAmount(distributable)} SOL`);

  // Generate a new note (not deposited yet - privateTransfer handles it!)
  const note = generateNote(transferAmount);
  console.log(`\n📝 Generated note (will be deposited automatically)`);
  console.log(`   Commitment: ${note.commitment.slice(0, 16)}...`);

  // Define recipients (up to 5)
  const recipient1 = Keypair.generate().publicKey;
  const recipient2 = Keypair.generate().publicKey;
  const recipient3 = Keypair.generate().publicKey;

  // Split distributable amount between recipients
  const amount1 = Math.floor(distributable * 0.5); // 50%
  const amount2 = Math.floor(distributable * 0.3); // 30%
  const amount3 = distributable - amount1 - amount2; // 20%

  console.log(`\n📋 Recipients:`);
  console.log(`   1. ${recipient1.toBase58().slice(0, 8)}... → ${formatAmount(amount1)} SOL`);
  console.log(`   2. ${recipient2.toBase58().slice(0, 8)}... → ${formatAmount(amount2)} SOL`);
  console.log(`   3. ${recipient3.toBase58().slice(0, 8)}... → ${formatAmount(amount3)} SOL`);

  // privateTransfer handles the complete flow:
  // 1. Deposits the note (since it's not deposited yet)
  // 2. Waits for confirmation
  // 3. Generates ZK proof
  // 4. Transfers to recipients via relay
  console.log(`\n🚀 Starting private transfer...`);
  const transferResult = await client.privateTransfer(
    connection,
    note, // Not deposited yet - privateTransfer handles it!
    [
      { recipient: recipient1, amount: amount1 },
      { recipient: recipient2, amount: amount2 },
      { recipient: recipient3, amount: amount3 },
    ],
    {
      onProgress: (status: string) => {
        console.log(`   ${status}`);
      },
    }
  );

  console.log(`\n✅ Transfer successful!`);
  console.log(`   Transaction: ${transferResult.signature}`);
  console.log(`   Nullifier: ${transferResult.nullifier.slice(0, 16)}...`);
  console.log(`   Root: ${transferResult.root.slice(0, 16)}...`);
  console.log(`\n📤 Outputs:`);
  transferResult.outputs.forEach((output: { recipient: string; amount: number }, i: number) => {
    console.log(`   ${i + 1}. ${output.recipient.slice(0, 8)}... → ${formatAmount(output.amount)} SOL`);
  });
}

main().catch((error) => {
  console.error("❌ Error:", error?.message ?? error);
  process.exit(1);
});

