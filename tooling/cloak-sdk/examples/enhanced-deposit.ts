/**
 * Enhanced Deposit Example
 * 
 * Demonstrates the new SDK features that replicate frontend functionality:
 * - Wallet adapter support
 * - V2.0 encrypted outputs
 * - Note scanning
 * - Enhanced progress tracking
 */

import {
  CloakSDK,
  generateCloakKeys,
  exportKeys,
  prepareEncryptedOutput,
  formatAmount,
  calculateFee,
  keypairToAdapter,
  CloakError,
} from "@cloak/sdk";
import { Connection, Keypair } from "@solana/web3.js";
import { readFileSync } from "fs";
import * as path from "path";

async function main() {
  console.log("🎯 Enhanced Cloak SDK Demo\n");
  
  // ============================================================================
  // Step 1: Generate and Save Cloak Keys (v2.0)
  // ============================================================================
  
  console.log("🔑 Generating Cloak keys (v2.0)...");
  const cloakKeys = generateCloakKeys();
  
  console.log("\n📦 Your keys (save these securely!):");
  console.log(exportKeys(cloakKeys));
  
  console.log("\n✅ Share this to receive encrypted notes:");
  console.log(`Public View Key: ${cloakKeys.view.pvk_hex}`);
  
  // ============================================================================
  // Step 2: Initialize SDK with Wallet
  // ============================================================================
  
  console.log("\n\n🔧 Initializing SDK...");
  
  // Load Solana keypair
  const homeDir = process.env.HOME || process.env.USERPROFILE;
  if (!homeDir) throw new Error("Could not determine home directory");
  
  const keypairPath = path.join(homeDir, ".config", "solana", "id.json");
  const secretKey = Uint8Array.from(JSON.parse(readFileSync(keypairPath, "utf8")));
  const keypair = Keypair.fromSecretKey(secretKey);
  
  // Convert keypair to wallet adapter format
  const wallet = keypairToAdapter(keypair);
  
  // Initialize SDK with wallet and Cloak keys
  const sdk = new CloakSDK({
    wallet,
    cloakKeys,
    network: "testnet",
  });
  
  console.log(`✅ SDK initialized`);
  console.log(`   Wallet: ${wallet.publicKey?.toBase58()}`);
  console.log(`   Network: testnet`);
  console.log(`   V2.0 Features: Enabled (scanning support)`);
  
  // ============================================================================
  // Step 3: Enhanced Deposit with Progress Tracking
  // ============================================================================
  
  console.log("\n\n📥 Depositing with enhanced tracking...");
  
  const connection = new Connection("https://api.testnet.solana.com", "confirmed");
  const amount = 100_000_000; // 0.1 SOL
  const fee = calculateFee(amount);
  
  console.log(`   Amount: ${formatAmount(amount)} SOL`);
  console.log(`   Fee: ${formatAmount(fee)} SOL`);
  
  try {
    const result = await sdk.deposit(connection, amount, {
      onProgress: (status, details) => {
        if (details) {
          const progressBar = "█".repeat(Math.floor((details.step! / details.totalSteps!) * 20));
          console.log(`   [${progressBar.padEnd(20)}] ${details.message}`);
        } else {
          console.log(`   ${status}`);
        }
      },
      onTransactionSent: (signature) => {
        console.log(`   📝 Transaction sent: ${signature}`);
      },
      onConfirmed: (signature, slot) => {
        console.log(`   ✅ Confirmed at slot ${slot}`);
      },
      computeUnits: 200_000,
      priorityFee: 1_000,
    });
    
    console.log(`\n✅ Deposit successful!`);
    console.log(`   Signature: ${result.signature}`);
    console.log(`   Leaf Index: ${result.leafIndex}`);
    console.log(`   Root: ${result.root.slice(0, 16)}...`);
    console.log(`\n💾 Note saved and ready to use`);
    
  } catch (error) {
    if (error instanceof CloakError) {
      console.error(`\n❌ ${error.category} error: ${error.message}`);
      console.error(`   Retryable: ${error.retryable}`);
      
      if (error.category === "indexer" && error.message.includes("duplicate")) {
        console.log(`\n💡 This deposit was already processed.`);
        console.log(`   The transaction succeeded but the indexer has it recorded.`);
        console.log(`   You can scan for notes to find it.`);
      }
    } else {
      console.error(`\n❌ Unexpected error:`, error);
    }
  }
  
  // ============================================================================
  // Step 4: Note Scanning
  // ============================================================================
  
  console.log("\n\n🔍 Scanning blockchain for your notes...");
  
  try {
    const scannedNotes = await sdk.scanNotes({
      onProgress: (current, total) => {
        const percent = Math.floor((current / total) * 100);
        const bar = "█".repeat(Math.floor(percent / 5));
        process.stdout.write(`\r   [${bar.padEnd(20)}] ${percent}% (${current}/${total})`);
      }
    });
    
    console.log(`\n\n✅ Scan complete!`);
    console.log(`   Found ${scannedNotes.length} notes`);
    
    if (scannedNotes.length > 0) {
      const totalBalance = scannedNotes.reduce((sum, note) => sum + note.amount, 0);
      console.log(`   Total balance: ${formatAmount(totalBalance)} SOL`);
      
      console.log(`\n📝 Your notes:`);
      scannedNotes.forEach((note, i) => {
        console.log(`   ${i + 1}. ${formatAmount(note.amount)} SOL - ${note.commitment.slice(0, 16)}...`);
      });
    }
    
  } catch (error) {
    if (error instanceof CloakError) {
      if (error.message.includes("requires Cloak keys")) {
        console.error(`\n❌ Scanning requires v2.0 keys`);
        console.log(`   Initialize SDK with: cloakKeys: generateCloakKeys()`);
      } else {
        console.error(`\n❌ Scan failed: ${error.message}`);
      }
    }
  }
  
  // ============================================================================
  // Step 5: Demonstrate V2.0 Features
  // ============================================================================
  
  console.log("\n\n🎯 V2.0 Features Summary:");
  console.log(`   ✓ Wallet adapter integration`);
  console.log(`   ✓ Deterministic key hierarchy`);
  console.log(`   ✓ Encrypted outputs for scanning`);
  console.log(`   ✓ Note scanning support`);
  console.log(`   ✓ Enhanced progress tracking`);
  console.log(`   ✓ Better error categorization`);
  console.log(`   ✓ Compute budget control`);
  
  console.log(`\n🚀 SDK is ready for production integration!`);
}

main().catch((error) => {
  console.error("\n❌ Fatal error:", error?.message ?? error);
  process.exit(1);
});

