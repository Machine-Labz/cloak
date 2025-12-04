use std::time::{Duration, Instant};

use blake3::Hasher;

/// Root metadata within the current acceptable window
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootMeta {
    pub root: [u8; 32],
    pub slot: u64,
}

/// Note metadata captured by indexer/scanner
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteMeta {
    pub commitment: [u8; 32],
    pub amount: u64,
    pub root: [u8; 32],
    pub leaf_index: u32,
}

/// Output type used for proof/output planning (address:32 || amount:u64)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub address: [u8; 32],
    pub amount: u64,
}

/// Selected note
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub note: NoteMeta,
}

pub mod orchestrator;

#[inline(always)]
pub fn calculate_fee(amount: u64) -> u64 {
    let fixed_fee = 2_500_000u64; // 0.0025 SOL
    let variable_fee = (amount.saturating_mul(5)) / 1_000; // 0.5%
    fixed_fee.saturating_add(variable_fee)
}

/// Select a single note to satisfy target_amount.
/// Heuristics: prefer largest anonymity bucket (same-amount group), then most recent root.
/// Notes must be under a root present in roots_window. Returns None if none can cover target.
pub fn select_note(
    target_amount: u64,
    roots_window: &[RootMeta],
    notes: &[NoteMeta],
) -> Option<Selected> {
    if target_amount == 0 {
        return None;
    }

    // Build root set for quick membership
    fn eq32(a: &[u8; 32], b: &[u8; 32]) -> bool {
        a == b
    }
    let root_set: Vec<[u8; 32]> = roots_window.iter().map(|r| r.root).collect();

    // Eligible notes: amount >= target_amount + fee (since fee deducted from amount to recipient)
    // Here we require note amount >= target_amount (recipient) + fee(note amount), conservative approximation via loop.
    let mut eligible: Vec<&NoteMeta> = Vec::new();
    'outer: for n in notes {
        // require root membership
        if !root_set.iter().any(|r| eq32(r, &n.root)) {
            continue;
        }
        // check conservation feasibility: recipient_amount = target_amount, total amount = n.amount
        let fee = calculate_fee(n.amount);
        if n.amount < target_amount.saturating_add(fee) {
            continue 'outer;
        }
        eligible.push(n);
    }
    if eligible.is_empty() {
        return None;
    }

    // Group by amount to compute anonymity buckets
    use std::collections::HashMap;
    let mut bucket_counts: HashMap<u64, usize> = HashMap::new();
    for n in &eligible {
        *bucket_counts.entry(n.amount).or_default() += 1;
    }

    // Pick note from bucket with largest count; tie-break by most recent root slot
    let mut best: Option<&NoteMeta> = None;
    let mut best_bucket_size: usize = 0;
    let mut best_slot: u64 = 0;

    // Build slot map for roots
    let mut root_slot = HashMap::<[u8; 32], u64>::new();
    for r in roots_window {
        root_slot.insert(r.root, r.slot);
    }

    for n in eligible {
        let b = *bucket_counts.get(&n.amount).unwrap_or(&0);
        let slot = *root_slot.get(&n.root).unwrap_or(&0);
        if b > best_bucket_size || (b == best_bucket_size && slot > best_slot) {
            best = Some(n);
            best_bucket_size = b;
            best_slot = slot;
        }
    }

    best.map(|n| Selected { note: n.clone() })
}

/// Compute single-output list and outputs_hash.
/// Hash = BLAKE3(address:32 || amount:u64_le)
pub fn compute_outputs_single(
    recipient_addr: [u8; 32],
    recipient_amount: u64,
) -> (Vec<Output>, [u8; 32]) {
    let outputs = vec![Output {
        address: recipient_addr,
        amount: recipient_amount,
    }];
    let mut hasher = Hasher::new();
    hasher.update(&recipient_addr);
    hasher.update(&recipient_amount.to_le_bytes());
    let h = hasher.finalize();
    let mut out_hash = [0u8; 32];
    out_hash.copy_from_slice(h.as_bytes());
    (outputs, out_hash)
}

/// Given a note total `amount`, compute (fee, recipient_amount) for single-output MVP
pub fn compute_fee_and_recipient_amount(amount: u64) -> (u64, u64) {
    let fee = calculate_fee(amount);
    let recipient_amount = amount.saturating_sub(fee);
    (fee, recipient_amount)
}

/// Build canonical 104-byte public inputs buffer: root||nf||outputs_hash||amount_le
pub fn build_public_inputs_104(
    root: &[u8; 32],
    nf: &[u8; 32],
    outputs_hash: &[u8; 32],
    amount: u64,
) -> [u8; 104] {
    let mut buf = [0u8; 104];
    buf[0..32].copy_from_slice(root);
    buf[32..64].copy_from_slice(nf);
    buf[64..96].copy_from_slice(outputs_hash);
    buf[96..104].copy_from_slice(&amount.to_le_bytes());
    buf
}

/// Randomized delay in wall-clock corresponding to 0–3 blocks.
/// Block time is configurable via env var RELAY_JITTER_BLOCK_MS (default 400ms).
pub fn jitter_delay(now: Instant) -> Duration {
    let _ = now; // reserved for potential seeding
    let blocks: u32 = fastrand::u32(0..=3);
    let block_ms: u64 = std::env::var("RELAY_JITTER_BLOCK_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(400);
    Duration::from_millis(blocks as u64 * block_ms)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_conservation_fee_and_outputs_hash() {
        // amount is note amount; recipient gets amount - fee
        let amount: u64 = 1_000_000_000; // 1 SOL
        let fee = calculate_fee(amount);
        let recipient_amount = amount - fee;
        let addr = [0x11u8; 32];
        let (outs, hash) = compute_outputs_single(addr, recipient_amount);
        assert_eq!(outs.len(), 1);
        assert_eq!(outs[0].address, addr);
        assert_eq!(outs[0].amount, recipient_amount);

        // Recompute hash like on-chain: H(addr||amount_le)
        let mut hasher = Hasher::new();
        hasher.update(&addr);
        hasher.update(&recipient_amount.to_le_bytes());
        let h2 = hasher.finalize();
        let mut chk = [0u8; 32];
        chk.copy_from_slice(h2.as_bytes());
        assert_eq!(
            hash, chk,
            "outputs_hash must match guest/on-chain recompute"
        );
        // Conservation
        let sum_outputs: u64 = outs.iter().map(|o| o.amount).sum();
        assert_eq!(sum_outputs + fee, amount);
    }

    #[test]
    fn test_build_public_inputs_and_fee() {
        let root = [0xAAu8; 32];
        let nf = [0xBBu8; 32];
        let outputs_hash = [0xCCu8; 32];
        let amount = 2_000_000_000u64;
        let (fee, recipient_amount) = compute_fee_and_recipient_amount(amount);
        assert_eq!(fee, calculate_fee(amount));
        assert_eq!(recipient_amount + fee, amount);

        let buf = build_public_inputs_104(&root, &nf, &outputs_hash, amount);
        assert_eq!(&buf[0..32], &root);
        assert_eq!(&buf[32..64], &nf);
        assert_eq!(&buf[64..96], &outputs_hash);
        let amt = u64::from_le_bytes(buf[96..104].try_into().unwrap());
        assert_eq!(amt, amount);
    }

    #[test]
    fn test_select_note_prefers_bucket_and_recent_root() {
        let roots = vec![
            RootMeta {
                root: [1u8; 32],
                slot: 100,
            },
            RootMeta {
                root: [2u8; 32],
                slot: 200,
            },
        ];
        let notes = vec![
            NoteMeta {
                commitment: [9u8; 32],
                amount: 10_000_000,
                root: [1u8; 32],
                leaf_index: 1,
            },
            NoteMeta {
                commitment: [8u8; 32],
                amount: 10_000_000,
                root: [2u8; 32],
                leaf_index: 2,
            },
            NoteMeta {
                commitment: [7u8; 32],
                amount: 20_000_000,
                root: [1u8; 32],
                leaf_index: 3,
            },
        ];
        // target recipient amount: choose a note where amount covers recipient+fee; both 10M and 20M could, but 10M bucket has size 2
        let sel = select_note(5_000_000, &roots, &notes).expect("selected");
        assert_eq!(sel.note.amount, 10_000_000);
        // tie within bucket: prefer most recent root slot=200
        assert_eq!(sel.note.root, [2u8; 32]);
    }

    #[test]
    fn test_jitter_delay_bounds() {
        std::env::set_var("RELAY_JITTER_BLOCK_MS", "500");
        let start = Instant::now();
        let d = jitter_delay(start);
        assert!(d <= Duration::from_millis(3 * 500));
    }
}
