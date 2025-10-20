# Merkle Tree & Proofs

- Binary tree, fixed height (e.g., 32).
- Leaves are commitments `C`.
- Indexer is append-only; new deposits are assigned `nextIndex` then included.

## API

- `GET /merkle/root` → `{ root: "<hex32>", nextIndex: <u32> }`
- `GET /merkle/proof/:index` → 
```

{
"pathElements": \["<hex32>", ...],  // bottom-up siblings
"pathIndices": \[0|1, ...]          // 0 = leaf was left, 1 = leaf was right
}

```

## Verification rule
At each level:
```

if pathIndices\[i] == 0:
parent = H( curr || pathElements\[i] )
else:
parent = H( pathElements\[i] || curr )

```
After last level, `parent` must equal the public `root`.

## Freshness
- Program keeps a **ring buffer** of recent roots (`K` latest).
- Withdraw must reference a root inside the buffer.