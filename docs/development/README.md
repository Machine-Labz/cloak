# Development Documentation

This directory contains development-specific documentation for the Cloak project.

## Files

- **[VKEY_FILE_SYSTEM.md](./VKEY_FILE_SYSTEM.md)** - File-based VKey hash management system
- **[CLEANUP_COMPLETE.md](./CLEANUP_COMPLETE.md)** - Project cleanup completion notes
- **[WORKSPACE_SETUP_COMPLETE.md](./WORKSPACE_SETUP_COMPLETE.md)** - Root workspace setup completion

## VKey Management

The VKey hash is now automatically managed using a file-based system:

- **Generation**: `make vkey-hash` or `cargo run -p vkey-generator --release`
- **Storage**: `target/vkey_hash.txt` (automatically created)
- **Usage**: Shield pool program reads from file at build time
- **Cleanup**: `make clean` removes all VKey files

## Project Structure

```
docs/
├── development/          # Development docs (this directory)
├── zk/                  # Zero-knowledge documentation
├── nonzk/               # Non-ZK service documentation
└── README.md            # Main documentation index
```

## Quick Commands

```bash
# Generate VKey and build everything
make build

# Just generate VKey hash
make vkey-hash

# Clean everything
make clean

# Run tests
make test
```

