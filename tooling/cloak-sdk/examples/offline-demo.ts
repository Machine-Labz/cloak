/**
 * Offline Cloak SDK Demo
 * 
 * This example demonstrates SDK features that work WITHOUT network connectivity:
 * - Fee calculations
 * - Note generation
 * - Key management (v2.0)
 * - Note serialization
 * - Cryptographic operations
 */

import {
  CloakSDK,
  formatAmount,
  calculateFee,
  getDistributableAmount,
  generateCloakKeys,
  generateNoteFromWallet,
  exportKeys,
  bytesToHex,
} from "@cloak/sdk";
import { Keypair } from "@solana/web3.js";

console.log("🎯 Cloak SDK Offline Demo\n");

// ============================================================================
// 1. Fee Calculations
// ============================================================================
console.log("📊 Fee Calculations:");

const amounts = [
  100_000_000,    // 0.1 SOL
  1_000_000_000,  // 1 SOL
  10_000_000_000, // 10 SOL
];

for (const amount of amounts) {
  const fee = calculateFee(amount);
  const distributable = getDistributableAmount(amount);
  
  console.log(`\n  Amount: ${formatAmount(amount)} SOL`);
  console.log(`  Fee: ${formatAmount(fee)} SOL`);
  console.log(`  Distributable: ${formatAmount(distributable)} SOL`);
}

// ============================================================================
// 2. Note Generation (v1.0 - Legacy)
// ============================================================================
console.log("\n\n📝 Note Generation (v1.0):");

const keypair = Keypair.generate();
const client = new CloakSDK({
  network: "devnet",
  keypairBytes: keypair.secretKey,
});

const note = client.generateNote(1_000_000_000);
console.log(`\n  Commitment: ${note.commitment.slice(0, 16)}...`);
console.log(`  Amount: ${formatAmount(note.amount)} SOL`);
console.log(`  Version: ${note.version}`);
console.log(`  Network: ${note.network}`);

// Serialize note
const noteJson = client.exportNote(note, true);
console.log(`\n  Serialized (first 150 chars):`);
console.log(`  ${noteJson.slice(0, 150)}...`);

// Parse it back
const parsedNote = client.parseNote(noteJson);
console.log(`\n  ✅ Successfully parsed back`);
console.log(`  Commitment matches: ${parsedNote.commitment === note.commitment}`);

// ============================================================================
// 3. Key Management (v2.0)
// ============================================================================
console.log("\n\n🔑 Key Management (v2.0):");

const keys = generateCloakKeys();
console.log(`\n  Master Seed: ${keys.master.seedHex.slice(0, 16)}...`);
console.log(`  Spend Key (secret): ${keys.spend.sk_spend_hex.slice(0, 16)}...`);
console.log(`  Spend Key (public): ${keys.spend.pk_spend_hex.slice(0, 16)}...`);
console.log(`  View Key (secret): ${keys.view.vk_secret_hex.slice(0, 16)}...`);
console.log(`  View Key (public): ${keys.view.pvk_hex.slice(0, 16)}...`);

// Export keys for backup
const keysBackup = exportKeys(keys);
console.log(`\n  📦 Keys Backup (first 200 chars):`);
console.log(`  ${keysBackup.slice(0, 200)}...`);

// Generate note using wallet keys (v2.0)
const v2Note = generateNoteFromWallet(keys, 500_000_000);
console.log(`\n  📝 v2.0 Note Generated:`);
console.log(`  Version: ${v2Note.version}`);
console.log(`  Commitment: ${v2Note.commitment.slice(0, 16)}...`);
console.log(`  Uses deterministic spend key: ${v2Note.sk_spend === keys.spend.sk_spend_hex}`);

// ============================================================================
// 4. Multi-Recipient Amount Distribution
// ============================================================================
console.log("\n\n💰 Multi-Recipient Distribution:");

const totalAmount = 1_000_000_000; // 1 SOL
const protocolFee = calculateFee(totalAmount);
const distributable = totalAmount - protocolFee;

// Split between 5 recipients
const splits = [0.3, 0.25, 0.2, 0.15, 0.1]; // 30%, 25%, 20%, 15%, 10%
console.log(`\n  Total: ${formatAmount(totalAmount)} SOL`);
console.log(`  Protocol Fee: ${formatAmount(protocolFee)} SOL`);
console.log(`  Distributable: ${formatAmount(distributable)} SOL`);
console.log(`\n  Distribution:`);

let calculatedTotal = 0;
const recipients = splits.map((split, index) => {
  const amount = Math.floor(distributable * split);
  calculatedTotal += amount;
  console.log(`    Recipient ${index + 1}: ${formatAmount(amount)} SOL (${split * 100}%)`);
  return { recipient: Keypair.generate().publicKey, amount };
});

// Handle rounding remainder
const remainder = distributable - calculatedTotal;
if (remainder > 0) {
  recipients[0].amount += remainder;
  console.log(`\n  ⚠️ Added ${remainder} lamports to recipient 1 (rounding remainder)`);
}

const finalTotal = recipients.reduce((sum, r) => sum + r.amount, 0);
console.log(`\n  ✅ Total distributed: ${formatAmount(finalTotal)} SOL`);
console.log(`  Matches distributable: ${finalTotal === distributable}`);

// ============================================================================
// 5. Cryptographic Demonstrations
// ============================================================================
console.log("\n\n🔐 Cryptographic Operations:");

import { generateCommitment, computeNullifier, computeOutputsHash, randomBytes } from "@cloak/sdk";

// Generate commitment
const amount = 1_000_000_000;
const r = randomBytes(32);
const skSpend = randomBytes(32);
const commitment = generateCommitment(amount, r, skSpend);

console.log(`\n  Commitment: ${bytesToHex(commitment).slice(0, 16)}...`);

// Compute nullifier
const leafIndex = 42;
const nullifier = computeNullifier(skSpend, leafIndex);
console.log(`  Nullifier: ${bytesToHex(nullifier).slice(0, 16)}...`);

// Compute outputs hash
const outputs = [
  { recipient: Keypair.generate().publicKey, amount: 500_000_000 },
  { recipient: Keypair.generate().publicKey, amount: 500_000_000 },
];
const outputsHash = computeOutputsHash(outputs);
console.log(`  Outputs Hash: ${bytesToHex(outputsHash).slice(0, 16)}...`);

// ============================================================================
// Summary
// ============================================================================
console.log("\n\n✅ Demo Complete!");
console.log("\n📚 What you learned:");
console.log("  ✓ Fee calculations and formatting");
console.log("  ✓ v1.0 note generation and serialization");
console.log("  ✓ v2.0 key hierarchy (master → spend → view)");
console.log("  ✓ Multi-recipient amount distribution");
console.log("  ✓ Cryptographic operations (commitments, nullifiers)");
console.log("\n💡 Next steps:");
console.log("  • Run tests: yarn test");
console.log("  • Review examples: ls examples/");
console.log("  • Read docs: cat README.md");
console.log("\n🚀 For live testing, ensure Cloak services are running!");

