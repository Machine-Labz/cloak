# SP1 Artifacts Directory

**Auto-generated files** - Do not commit binary artifacts to git.

## How it works

- **Development**: Placeholder artifacts created automatically on first API call
- **Tests**: Test artifacts generated during test execution  
- **Production**: Real SP1 guest ELF and verification keys deployed separately

## File Structure

```
artifacts/
├── v2.0.0/
│   ├── guest.elf           # SP1 guest program (auto-generated)
│   └── verification.key    # ZK verification key (auto-generated)
└── vX.Y.Z/                 # Other versions...
```

## API Endpoints

- `GET /api/v1/artifacts/withdraw/:version` - Get artifact metadata + hashes
- `GET /api/v1/artifacts/files/:version/:filename` - Download artifact files

All artifacts include SHA-256 integrity hashes for verification.
