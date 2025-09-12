# Withdraw Circuit (SP1)

**Private**: `amount:u64, r:32, sk_spend:32, leaf_index:u32, merkle_path{pathElements[], pathIndices[]}`  
**Public**: `root:32, nf:32, fee_bps:u16, outputs_hash:32, amount:u64`

**Constraints**  
1. `pk_spend = H(sk_spend)`  
2. `C = H(amount || r || pk_spend)`  
3. `MerkleVerify(C, merkle_path) == root`  
4. `nf == H(sk_spend || leaf_index)`  
5. `sum(outputs) + fee(amount, fee_bps) == amount`  
6. `H(serialize(outputs)) == outputs_hash`

