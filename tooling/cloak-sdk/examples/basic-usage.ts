/**
 * Basic Cloak SDK Usage Example
 *
 * This example demonstrates:
 * 1. Initializing the Cloak client
 * 2. Depositing SOL to create a private note
 * 3. Performing a private transfer to multiple recipients
 * 4. Withdrawing to a single address
 */

import { CloakSDK, formatAmount, calculateFee } from "@cloak/sdk";
import { Connection, Keypair } from "@solana/web3.js";
import { readFileSync } from "fs";
import * as path from "path";

async function main() {
  // ============================================================================
  // STEP 1: Initialize Cloak Client
  // ============================================================================

  // Generate a keypair for the SDK to use
  // In a real app, you'd load this from your wallet or secure storage

  // Safely get the user's home directory
  const homeDir = process.env.HOME || process.env.USERPROFILE;
  if (!homeDir) throw new Error("Could not determine the user's home directory for keypair loading.");

  const systemKeypairPath = path.join(homeDir, ".config", "solana", "id.json");
  const secretKeyString = readFileSync(systemKeypairPath, "utf8");
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  const keypair = Keypair.fromSecretKey(secretKey);
  const connection = new Connection("https://api.testnet.solana.com", "confirmed");

  // Initialize SDK - storage is optional (defaults to in-memory)
  // For persistence, pass a StorageAdapter: storage: new LocalStorageAdapter()
  const client = new CloakSDK({
    network: "testnet",
    keypairBytes: keypair.secretKey,
    // storage: new MemoryStorageAdapter() // Optional - in-memory by default
  });

  console.log("✅ Cloak client initialized");
  console.log(`   Using keypair: ${keypair.publicKey.toBase58()}`);

  // ============================================================================
  // STEP 2: Deposit SOL
  // ============================================================================

  const depositAmount = 10_000_000; // 0.01 SOL in lamports
  const fee = calculateFee(depositAmount);

  console.log(`\n📥 Depositing ${formatAmount(depositAmount)} SOL...`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);

  const depositResult = await client.deposit(
    connection,
    depositAmount,
    {
      onProgress: (status: string) => {
        console.log(`   ${status}`);
      },
    }
  );

  console.log(`✅ Deposit successful!`);
  console.log(`   Transaction: ${depositResult.signature}`);
  console.log(`   Leaf index: ${depositResult.leafIndex}`);
  console.log(`   Root: ${depositResult.root.slice(0, 16)}...`);

  // Save the note securely!
  const note = depositResult.note;
  console.log(`\n💾 Save this note securely:`);
  console.log(client.exportNote(note, true));

  // ============================================================================
  // STEP 3: Private Transfer (Complete Flow: Deposit + Withdraw)
  // ============================================================================

  // This demonstrates the main use case: privately send funds to recipients
  // privateTransfer handles EVERYTHING - deposit, wait, proof, transfer!

  console.log(`\n💸 Private transfer to 3 recipients (complete flow):`);

  // Generate a new note (not deposited yet)
  const newNote = client.generateNote(depositAmount);

  // Define recipients (up to 5)
  // In a real app, these would be actual recipient addresses
  // For this example, we'll generate some demo addresses
  const recipient1 = Keypair.generate().publicKey;
  const recipient2 = Keypair.generate().publicKey;

  // Calculate distributable amount (after protocol fees)
  const distributable = depositAmount - fee;

  // Split between recipients
  const amount1 = Math.floor(distributable * 0.8); // 80%
  const amount2 = distributable - amount1;

  console.log(`   Recipient 1 (${recipient1.toBase58().slice(0, 8)}...): ${formatAmount(amount1)} SOL`);
  console.log(`   Recipient 2 (${recipient2.toBase58().slice(0, 8)}...): ${formatAmount(amount2)} SOL`);

  // privateTransfer handles the complete flow:
  // 1. Deposits the note (since it's not deposited yet)
  // 2. Waits for confirmation
  // 3. Generates ZK proof
  // 4. Transfers to recipients
  const transferResult = await client.privateTransfer(
    connection,
    newNote, // Not deposited yet - privateTransfer handles it!
    [
      { recipient: recipient1, amount: amount1 },
      { recipient: recipient2, amount: amount2 },
    ],
    {
      onProgress: (status: string) => {
        console.log(`   ${status}`);
      },
    }
  );

  console.log(`✅ Transfer successful!`);
  console.log(`   Transaction: ${transferResult.signature}`);
  console.log(`   Nullifier: ${transferResult.nullifier.slice(0, 16)}...`);

  // ============================================================================
  // STEP 4: Using a Previously Deposited Note
  // ============================================================================

  // If you already deposited a note (like in STEP 3), you can use it directly
  console.log(`\n💸 Withdrawing previously deposited note...`);

  // In a real app, this would be an actual recipient address
  const recipientSingle = Keypair.generate().publicKey;

  const withdrawResult = await client.withdraw(
    connection,
    note, // This note was already deposited in STEP 3
    recipientSingle,
    {
      withdrawAll: true, // Withdraw full amount minus fees
      onProgress: (status: string) => console.log(`   ${status}`),
    }
  );

  console.log(`✅ Withdrawal successful!`);
  console.log(`   Transaction: ${withdrawResult.signature}`);

  // ============================================================================
  // STEP 5: Working with Notes
  // ============================================================================

  console.log(`\n📝 Note management:`);

  // Export note as JSON
  const noteJson = client.exportNote(note, true);
  console.log(`   Exported: ${noteJson.slice(0, 100)}...`);

  // Parse note from JSON
  const parsedNote = client.parseNote(noteJson);
  console.log(`   Parsed commitment: ${parsedNote.commitment.slice(0, 16)}...`);

  // Check if note is withdrawable
  const withdrawable = client.isWithdrawable(note);
  console.log(`   Withdrawable: ${withdrawable}`);

  // Get Merkle proof
  const proof = await client.getMerkleProof(note.leafIndex!);
  console.log(`   Merkle proof depth: ${proof.pathElements.length}`);

  // Get current root
  const currentRoot = await client.getCurrentRoot();
  console.log(`   Current root: ${currentRoot.slice(0, 16)}...`);

  console.log(`\n🎉 Example complete!`);
}

// Run the example
main().catch((error) => {
  console.error("❌ Error:", error?.message ?? error);
  throw error;
});
