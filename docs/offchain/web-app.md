---
title: Web Application
description: Next.js application providing deposit/withdraw UX, admin utilities, and note management helpers.
---

# Web Application

The Cloak web client lives in [`services/web`](https://github.com/cloak-labz/cloak/tree/main/services/web). It is a Next.js 14 app (App Router) with Tailwind CSS, wallet adapter integration, and WASM helpers for proof workflows.

## Features

- **Transaction Hub (`/transaction`)** – Tabbed interface for deposits and withdrawals with wallet connection via `@solana/wallet-adapter-react`.
- **Privacy Demo (`/privacy-demo`)** – Walk-through describing how private notes work with interactive explanations.
- **Admin Console (`/admin`)** – Tooling to verify/create program-derived accounts (pool, roots ring, nullifier shard, treasury) directly from the browser.
- **WASM Prover Playground (`/wasm-test`)** – Experiments with in-browser proving using the `wasm-prover/` bundle.
- **Note Manager Utilities** – `lib/note-manager.ts` handles note generation, serialization, encryption placeholders, file download/upload, and fee calculations.

## Technology Stack

- Next.js App Router with React Server & Client Components.
- Tailwind CSS + shadcn/ui component primitives.
- Wallet adapter UI package for Phantom/Solana wallets.
- `lucide-react` icon set, `sonner` toasts for feedback.
- TypeScript strict mode enabled via `tsconfig.json`.

## Configuration

Environment variables (via `.env.local`):

```
NEXT_PUBLIC_SOLANA_RPC_URL=http://127.0.0.1:8899
NEXT_PUBLIC_PROGRAM_ID=<shield-pool-program-id>
NEXT_PUBLIC_POOL_ADDRESS=<pool-pda>
NEXT_PUBLIC_ROOTS_RING_ADDRESS=<roots-ring-pda>
NEXT_PUBLIC_NULLIFIER_SHARD_ADDRESS=<nullifier-pda>
NEXT_PUBLIC_TREASURY_ADDRESS=<treasury-pda>
NEXT_PUBLIC_INDEXER_URL=http://localhost:3001
NEXT_PUBLIC_RELAY_URL=http://localhost:3002
```

These values power the admin page and API calls.

## Folder Highlights

- `app/` – Route segments for public, admin, and experimental pages.
- `components/transaction/*` – Deposit and withdraw flow UIs composed of form controls, preview cards, and status indicators.
- `hooks/` – Custom React hooks for local storage, wallet state, and environment binding.
- `lib/note-manager.ts` – Core logic for note generation (`generateNote`), persistence (`saveNote`, `loadWithdrawableNotes`), fee calculation, and outputs hashing.
- `wasm-prover/` – Bundled WASM assets and loader for client-side proving experiments.

## Development

```bash
cd services/web
npm install
npm run dev
```

The app listens on `http://localhost:3000`. Wallet adapter requires running in a secure context; use `localhost` or configure HTTPS for remote access.

## Testing & Linting

- `npm run lint` – ESLint with Next.js defaults.
- `npm run test` – If configured (currently placeholder; add Jest/Playwright as needed).

## Integration Tips

- Connect to the same Solana cluster as the relay/indexer (set `NEXT_PUBLIC_SOLANA_RPC_URL`).
- Use the admin console after deploying programs to create PDAs via wallet instructions.
- The note manager assumes BLAKE3 hashing and little-endian amount encoding; align with the SP1 circuit.
- For production, replace placeholder encryption with audited note encryption (the current implementation is demonstrative).

Refer to [`docs/nonzk/frontend.md`](../nonzk/frontend.md) for legacy frontend notes and historical context.
