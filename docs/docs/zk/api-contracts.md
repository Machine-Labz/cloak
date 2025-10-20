# API Contracts (Indexer & Relay)

## Indexer
- `GET /merkle/root` → `{ root, nextIndex }`
- `GET /merkle/proof/:index` → `{ pathElements[], pathIndices[] }`
- `GET /notes/range?start=<n>&end=<n>` → 
  `{ encrypted_outputs: ["hex",...], hasMore, total, start, end }`
- `GET /artifacts/withdraw/:version` →
  `{ guestElfUrl, vk, sha256:{elf,vk}, sp1Version }`

> No `senderAddress` filter to avoid metadata leakage.

## Relay
- `POST /withdraw`
```

{
"outputs":\[{"address":"<base58>","amount":"<lamports>"}],
"policy":{"fee\_bps":60},
"publicInputs":{
"root":"<hex32>",
"nf":"<hex32>",
"amount":"<u64>",
"fee\_bps":60,
"outputs\_hash":"<hex32>"
},
"proofBytes":"<base64>"
}

```
→ `200 { requestId, txid, rootUsed, nf, receiptAsset }`

- `GET /status/:requestId` → 
`{ state:"queued|executing|settled|failed", txid?, rootUsed?, nf?, receiptAsset? }`
