//! Example: Batch commitment hash computation
//!
//! Demonstrates computing batch_hash for PoW claims.
//!
//! Run with: cargo run --package relay --example batch_commitment

use relay::miner::{compute_batch_hash, compute_single_job_hash};

fn main() {
    println!("=== Batch Commitment Example ===\n");

    // Example 1: Single job (k=1)
    println!("1. Single job batch (k=1):");
    let job_id = "withdraw-550e8400-e29b-41d4-a716-446655440000";
    let hash = compute_single_job_hash(job_id);
    println!("   Job ID: {}", job_id);
    println!("   Batch hash: {:x?}...\n", &hash[0..8]);

    // Example 2: Multiple jobs (k=3)
    println!("2. Multi-job batch (k=3):");
    let job_ids = vec![
        "withdraw-001".to_string(),
        "withdraw-002".to_string(),
        "withdraw-003".to_string(),
    ];
    let hash = compute_batch_hash(&job_ids);
    println!("   Jobs: {:?}", job_ids);
    println!("   Batch hash: {:x?}...\n", &hash[0..8]);

    // Example 3: Order matters
    println!("3. Order sensitivity:");
    let batch_a = vec!["job-A".to_string(), "job-B".to_string()];
    let batch_b = vec!["job-B".to_string(), "job-A".to_string()];

    let hash_a = compute_batch_hash(&batch_a);
    let hash_b = compute_batch_hash(&batch_b);

    println!("   Batch A: {:?}", batch_a);
    println!("   Hash A:  {:x?}...", &hash_a[0..8]);
    println!("   Batch B: {:?}", batch_b);
    println!("   Hash B:  {:x?}...", &hash_b[0..8]);
    println!("   Hashes equal? {}\n", hash_a == hash_b);

    // Example 4: Deterministic
    println!("4. Determinism check:");
    let jobs = vec!["test-1".to_string(), "test-2".to_string()];
    let hash1 = compute_batch_hash(&jobs);
    let hash2 = compute_batch_hash(&jobs);
    println!("   Jobs: {:?}", jobs);
    println!("   Hash 1: {:x?}...", &hash1[0..8]);
    println!("   Hash 2: {:x?}...", &hash2[0..8]);
    println!("   Deterministic? {}\n", hash1 == hash2);

    println!("=== Example Complete ===");
    println!("\nKey takeaways:");
    println!("- Batch hash binds PoW claim to specific jobs");
    println!("- Order of job IDs matters (different order = different hash)");
    println!("- Hashing is deterministic (same input = same output)");
    println!("- Used in Claim PDA derivation and anti-replay checks");
}
