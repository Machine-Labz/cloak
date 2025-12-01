/**
 * Deposit Example
 *
 * Demonstrates how to deposit SOL into the Cloak protocol to create a private note.
 * The note can then be used for private transfers, withdrawals, or swaps.
 */

import { CloakSDK, formatAmount, calculateFee, getShieldPoolPDAs, exportNote } from "@cloaklabz/sdk";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
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


  // Check if pool is initialized
  const programId = new PublicKey("c1oak6tetxYnNfvXKFkpn1d98FxtK7B68vBQLYQpWKp");
  const mintForSOL = new PublicKey(new Uint8Array(32)); // 32 zero bytes for native SOL
  const { pool: poolAddress } = getShieldPoolPDAs(programId, mintForSOL);
  console.log(`\n🔍 Checking pool account: ${poolAddress.toBase58()}`);
  
  try {
    const poolAccount = await connection.getAccountInfo(poolAddress);
    if (!poolAccount) {
      console.log(`   ⚠️  Pool account not found on devnet!`);
      console.log(`   The pool needs to be initialized before deposits can work.`);
      process.exit(1);
    } else if (poolAccount.owner.toBase58() !== programId.toBase58()) {
      console.log(`   ⚠️  Pool account owner mismatch!`);
      process.exit(1);
    } else {
      console.log(`   ✅ Pool account exists and is owned by the program`);
    }
  } catch (error: any) {
    console.log(`   ⚠️  Error checking pool: ${error.message}`);
    console.log(`   Continuing anyway...`);
  }

  // Deposit SOL
  const depositAmount = 10_000_000; // 0.01 SOL in lamports
  const fee = calculateFee(depositAmount);

  console.log(`\n📥 Depositing ${formatAmount(depositAmount)} SOL...`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);
  console.log(`   You'll receive: ${formatAmount(depositAmount - fee)} SOL in your private note`);

  const depositResult = await client.deposit(
    connection,
    depositAmount,
    {
      onProgress: (status: string) => {
        console.log(`   ${status}`);
      },
    }
  );

  console.log(`\n✅ Deposit successful!`);
  console.log(`   Transaction: ${depositResult.signature}`);
  console.log(`   Leaf index: ${depositResult.leafIndex}`);
  console.log(`   Root: ${depositResult.root.slice(0, 16)}...`);

  // Save the note securely!
  const note = depositResult.note;
  console.log(`\n💾 Save this note securely - you'll need it to withdraw:`);
  console.log(exportNote(note, true));

  console.log(`\n📝 Next steps:`);
  console.log(`   - Use this note with withdraw-example.ts to withdraw funds`);
  console.log(`   - Use this note with transfer-example.ts to send to multiple recipients`);
  console.log(`   - Use this note with swap-example.ts to swap for tokens`);
}

main().catch((error) => {
  console.error("❌ Error:", error?.message ?? error);
  process.exit(1);
});

