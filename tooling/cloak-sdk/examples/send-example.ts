/**
 * Send Example
 *
 * Demonstrates the send() convenience method - a simpler API wrapper around privateTransfer.
 * Use this when you want a cleaner interface for sending SOL to multiple recipients.
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

  // Amount to send
  const sendAmount = 10_000_000; // 0.01 SOL in lamports
  const fee = calculateFee(sendAmount);
  const distributable = sendAmount - fee;

  console.log(`\n💸 Using send() method (convenience wrapper)`);
  console.log(`   Total amount: ${formatAmount(sendAmount)} SOL`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
  console.log(`   Distributable: ${formatAmount(distributable)} SOL`);

  // Generate a new note
  const note = client.generateNote(sendAmount);
  console.log(`\n📝 Generated note (will be deposited automatically)`);

  // Define recipients
  const recipient1 = Keypair.generate().publicKey;
  const recipient2 = Keypair.generate().publicKey;

  // Split amount
  const amount1 = Math.floor(distributable * 0.6);
  const amount2 = distributable - amount1;

  console.log(`\n📋 Sending to 2 recipients:`);
  console.log(`   1. ${recipient1.toBase58().slice(0, 8)}... → ${formatAmount(amount1)} SOL`);
  console.log(`   2. ${recipient2.toBase58().slice(0, 8)}... → ${formatAmount(amount2)} SOL`);

  // send() is a convenience wrapper around privateTransfer
  // It provides a simpler, more intuitive API
  console.log(`\n🚀 Starting send...`);
  const sendResult = await client.send(
    connection,
    note,
    [
      { recipient: recipient1, amount: amount1 },
      { recipient: recipient2, amount: amount2 },
    ],
    {
      onProgress: (status: string) => console.log(`   ${status}`),
    }
  );

  console.log(`\n✅ Send successful!`);
  console.log(`   Transaction: ${sendResult.signature}`);
  console.log(`   Outputs: ${sendResult.outputs.length} recipients`);
  console.log(`\n📤 Sent:`);
  sendResult.outputs.forEach((output, i) => {
    console.log(`   ${i + 1}. ${output.recipient.slice(0, 8)}... → ${formatAmount(output.amount)} SOL`);
  });
}

main().catch((error) => {
  console.error("❌ Error:", error?.message ?? error);
  process.exit(1);
});

