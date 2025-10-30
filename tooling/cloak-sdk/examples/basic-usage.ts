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
import { Connection, PublicKey, Keypair } from "@solana/web3.js";

async function main() {
  // ============================================================================
  // STEP 1: Initialize Cloak Client
  // ============================================================================

  const client = new CloakSDK({
    network: "devnet",
    programId: new PublicKey("YOUR_PROGRAM_ID"),
    poolAddress: new PublicKey("YOUR_POOL_ADDRESS"),
    commitmentsAddress: new PublicKey("YOUR_COMMITMENTS_ADDRESS"),
    // Option A: Single API URL proxying indexer+relay
    apiUrl: "https://api.your-cloak.example.com",
    // Option B: Separate services
    // indexerUrl: "https://your-indexer-url.com",
    // relayUrl: "https://your-relay-url.com",
    // Optional: custom proof timeout
    proofTimeout: 5 * 60 * 1000, // 5 minutes
  });

  console.log("✅ Cloak client initialized");

  // ============================================================================
  // STEP 2: Connect to Solana
  // ============================================================================

  const connection = new Connection("https://api.devnet.solana.com", "confirmed");

  // In a real app, you'd use a wallet adapter
  // For this example, we'll use a keypair
  const payer = Keypair.generate(); // Replace with your wallet

  // Mock wallet object with sendTransaction method
  const wallet = {
    publicKey: payer.publicKey,
    sendTransaction: async (tx: any, connection: Connection) => {
      // Sign and send
      tx.partialSign(payer);
      return await connection.sendRawTransaction(tx.serialize());
    },
  };

  console.log(`✅ Connected to Solana (${payer.publicKey.toBase58()})`);

  // ============================================================================
  // STEP 3: Deposit SOL
  // ============================================================================

  const depositAmount = 1_000_000_000; // 1 SOL in lamports
  const fee = calculateFee(depositAmount);

  console.log(`\n📥 Depositing ${formatAmount(depositAmount)} SOL...`);
  console.log(`   Protocol fee: ${formatAmount(fee)} SOL`);

  const depositResult = await client.deposit(
    connection,
    wallet,
    depositAmount,
    {
      onProgress: (status) => {
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
  // STEP 4: Private Transfer (Complete Flow: Deposit + Withdraw)
  // ============================================================================

  // This demonstrates the main use case: privately send funds to recipients
  // privateTransfer handles EVERYTHING - deposit, wait, proof, transfer!

  console.log(`\n💸 Private transfer to 3 recipients (complete flow):`);

  // Generate a new note (not deposited yet)
  const newNote = client.generateNote(depositAmount);

  // Define recipients (up to 5)
  const recipient1 = new PublicKey("RECIPIENT_ADDRESS_1");
  const recipient2 = new PublicKey("RECIPIENT_ADDRESS_2");
  const recipient3 = new PublicKey("RECIPIENT_ADDRESS_3");

  // Calculate distributable amount (after protocol fees)
  const distributable = depositAmount - fee;

  // Split between recipients
  const amount1 = Math.floor(distributable * 0.5); // 50%
  const amount2 = Math.floor(distributable * 0.3); // 30%
  const amount3 = distributable - amount1 - amount2; // Remaining 20%

  console.log(`   Recipient 1: ${formatAmount(amount1)} SOL`);
  console.log(`   Recipient 2: ${formatAmount(amount2)} SOL`);
  console.log(`   Recipient 3: ${formatAmount(amount3)} SOL`);

  // privateTransfer handles the complete flow:
  // 1. Deposits the note (since it's not deposited yet)
  // 2. Waits for confirmation
  // 3. Generates ZK proof
  // 4. Transfers to recipients
  const transferResult = await client.privateTransfer(
    connection,
    wallet,
    newNote, // Not deposited yet - privateTransfer handles it!
    [
      { recipient: recipient1, amount: amount1 },
      { recipient: recipient2, amount: amount2 },
      { recipient: recipient3, amount: amount3 },
    ],
    {
      relayFeeBps: 50, // 0.5% relay fee
      onProgress: (status) => {
        console.log(`   ${status}`);
      },
      onProofProgress: (progress) => {
        // Update every 10%
        if (progress % 10 === 0) {
          console.log(`   Proof generation: ${progress}%`);
        }
      },
    }
  );

  console.log(`✅ Transfer successful!`);
  console.log(`   Transaction: ${transferResult.signature}`);
  console.log(`   Nullifier: ${transferResult.nullifier.slice(0, 16)}...`);

  // ============================================================================
  // STEP 5: Using a Previously Deposited Note
  // ============================================================================

  // If you already deposited a note (like in STEP 3), you can use it directly
  console.log(`\n💸 Withdrawing previously deposited note...`);

  const recipientSingle = new PublicKey("RECIPIENT_ADDRESS");

  const withdrawResult = await client.withdraw(
    connection,
    wallet,
    note, // This note was already deposited in STEP 3
    recipientSingle,
    {
      withdrawAll: true, // Withdraw full amount minus fees
      relayFeeBps: 50,
      onProgress: (status) => console.log(`   ${status}`),
    }
  );

  console.log(`✅ Withdrawal successful!`);
  console.log(`   Transaction: ${withdrawResult.signature}`);

  // ============================================================================
  // STEP 6: Working with Notes
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
