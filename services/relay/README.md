# Relay service

- `POST /withdraw` -> submit tx, confirm, return `txid`
- Simple FIFO queue; safe retries; error mapping
- (Optional) Mint receipt NFT after settlement
