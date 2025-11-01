//! Batch commitment logic
//!
//! Computes batch_hash = BLAKE3(job_ids) for PoW claims.
//! Each claim covers k withdrawals identified by their job IDs.

use blake3::Hasher;

/// Compute batch commitment hash from job IDs
///
/// The batch_hash is used to:
/// 1. Bind a PoW claim to specific jobs
/// 2. Prevent claim reuse for different jobs
/// 3. Derive unique Claim PDA per batch
///
/// # Arguments
/// * `job_ids` - List of job IDs to include in batch (must be non-empty)
///
/// # Returns
/// 32-byte BLAKE3 hash of all job IDs
///
/// # Example
/// ```
/// use cloak_miner::batch::compute_batch_hash;
///
/// let job_ids = vec!["job-001".to_string(), "job-002".to_string()];
/// let batch_hash = compute_batch_hash(&job_ids);
/// assert_eq!(batch_hash.len(), 32);
/// ```
pub fn compute_batch_hash(job_ids: &[String]) -> [u8; 32] {
    let mut hasher = Hasher::new();

    // Hash each job ID in order
    for job_id in job_ids {
        hasher.update(job_id.as_bytes());
    }

    *hasher.finalize().as_bytes()
}

/// Compute batch hash for a single job (k=1)
///
/// Convenience wrapper for single-job batches.
pub fn compute_single_job_hash(job_id: &str) -> [u8; 32] {
    compute_batch_hash(&[job_id.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_batch_hash_single() {
        let job_id = "test-job-123".to_string();
        let hash = compute_batch_hash(&[job_id.clone()]);

        // Should produce 32-byte hash
        assert_eq!(hash.len(), 32);

        // Should be deterministic
        let hash2 = compute_batch_hash(&[job_id]);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_compute_batch_hash_multiple() {
        let job_ids = vec![
            "job-001".to_string(),
            "job-002".to_string(),
            "job-003".to_string(),
        ];

        let hash = compute_batch_hash(&job_ids);
        assert_eq!(hash.len(), 32);

        // Should be deterministic
        let hash2 = compute_batch_hash(&job_ids);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_batch_hash_order_matters() {
        let batch1 = vec!["job-A".to_string(), "job-B".to_string()];
        let batch2 = vec!["job-B".to_string(), "job-A".to_string()];

        let hash1 = compute_batch_hash(&batch1);
        let hash2 = compute_batch_hash(&batch2);

        // Order matters - different order = different hash
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_batch_hash_changes_with_content() {
        let batch1 = vec!["job-001".to_string()];
        let batch2 = vec!["job-002".to_string()];

        let hash1 = compute_batch_hash(&batch1);
        let hash2 = compute_batch_hash(&batch2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_single_job_hash_convenience() {
        let job_id = "convenience-test";

        let hash1 = compute_single_job_hash(job_id);
        let hash2 = compute_batch_hash(&[job_id.to_string()]);

        // Should produce same result
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_empty_batch() {
        // Edge case: empty batch
        let empty: Vec<String> = vec![];
        let hash = compute_batch_hash(&empty);

        // Should still produce valid hash (hash of empty input)
        assert_eq!(hash.len(), 32);

        // Should be deterministic
        let hash2 = compute_batch_hash(&empty);
        assert_eq!(hash, hash2);

        // Should be different from any non-empty batch
        let non_empty = vec!["job-001".to_string()];
        let non_empty_hash = compute_batch_hash(&non_empty);
        assert_ne!(hash, non_empty_hash);
    }

    #[test]
    fn test_batch_hash_with_special_chars() {
        let job_ids = vec![
            "job-with-uuid-550e8400-e29b-41d4-a716-446655440000".to_string(),
            "job/with/slashes".to_string(),
            "job with spaces".to_string(),
        ];

        let hash = compute_batch_hash(&job_ids);
        assert_eq!(hash.len(), 32);

        // Should be reproducible
        let hash2 = compute_batch_hash(&job_ids);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_large_batch() {
        // Test with max_k sized batch (e.g., 100 jobs)
        let job_ids: Vec<String> = (0..100).map(|i| format!("job-{:04}", i)).collect();

        let hash = compute_batch_hash(&job_ids);
        assert_eq!(hash.len(), 32);

        // Should be deterministic even with large batch
        let hash2 = compute_batch_hash(&job_ids);
        assert_eq!(hash, hash2);
    }
}
