# Merkle Tree (Append-only)

- Binary tree, fixed height (e.g., 32)
- Leaves = commitments `C`
- Indexer exposes:
  - `GET /merkle/root` → `{ root:<hex32>, nextIndex:<u32> }`
  - `GET /merkle/proof/:index` → `{ pathElements:[hex32], pathIndices:[0|1] }`
  - `GET /notes/range?start=&end=` → `{ encrypted_outputs:["hex",...], total, hasMore }`
- Verification per level:
```
if bit == 0: parent = H(curr || sibling)
else:        parent = H(sibling || curr)
```
- Program keeps a **ring buffer** of recent roots; withdraw must use a root in the buffer.

