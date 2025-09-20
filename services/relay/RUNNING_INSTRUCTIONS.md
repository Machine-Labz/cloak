# 🚀 Cloak Relay Service - Running Instructions

This guide provides complete instructions for setting up and running the Cloak Relay Service.

## 📋 Prerequisites

### System Requirements
- **Operating System**: Linux, macOS, or Windows (Linux recommended)
- **Rust**: Latest stable version (1.88.0 or later)
- **Memory**: At least 4GB RAM
- **Storage**: At least 2GB free space

### Required Services (for production mode)
- **PostgreSQL**: Version 15 or later
- **Redis**: Version 7 or later
- **Docker**: For running dependencies (optional but recommended)

## 🔧 Setup Instructions

### 1. Clone and Navigate
```bash
git clone <repository-url>
cd cloak/services/relay
```

### 2. Environment Setup

#### Option A: Using Direct Rust Toolchain (Recommended)
If you encounter proxy issues with rustup, use the direct toolchain path:
```bash
# Use the direct path to avoid proxy issues
export CARGO_PATH="/home/$USER/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo"
```

#### Option B: Standard Rust Setup
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Update to latest stable
rustup update stable
```

## 🏃‍♂️ Running the Service

### Quick Start (Basic Mode)
The service can run in basic mode without external dependencies for testing:

```bash
# Navigate to relay directory
cd services/relay

# Run with direct cargo path (if proxy issues)
RUST_LOG=info /home/$USER/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo run

# OR with standard cargo (if no proxy issues)
RUST_LOG=info cargo run
```

The service will start on **port 3002** and display:
```
INFO relay: Starting relay service in basic mode
INFO relay: Relay service listening on 0.0.0.0:3002
```

### Production Mode (with PostgreSQL and Redis)
For full functionality, you'll need to set up the external services:

#### 1. Start Dependencies
```bash
# Start PostgreSQL and Redis using Docker
docker-compose up -d
```

#### 2. Set Environment Variables
```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/relay"
export REDIS_URL="redis://localhost:6379"
export RUST_LOG=info
```

#### 3. Run the Service
```bash
cargo run --release
```

## 🧪 Testing the Service

### Manual Testing
```bash
# Test health endpoint
curl http://localhost:3002/health

# Expected response:
# {"status":"ok","message":"Relay service is healthy"}

# Test root endpoint
curl http://localhost:3002/

# Expected response:
# {"service":"Cloak Relay","version":"0.1.0","status":"running"}
```

### Using the Test Script
```bash
# Make the test script executable
chmod +x examples/test_api.sh

# Run the comprehensive test
./examples/test_api.sh
```

The test script will:
1. ✅ Check service health
2. ✅ Test the withdraw endpoint (with mock data)
3. ✅ Display example request/response formats

## 🐛 Troubleshooting

### Common Issues

#### 1. Rust Proxy Errors
```
error: unknown proxy name: 'Cursor-1.2.2-x86_64'
```
**Solution**: Use the direct cargo path:
```bash
/home/$USER/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo run
```

#### 2. Port Already in Use
```
Error: Address already in use (os error 98)
```
**Solution**: Kill existing processes or use a different port:
```bash
# Kill existing relay processes
pkill -f "target/debug/relay"

# Or find and kill the specific process
lsof -ti:3002 | xargs kill -9
```

#### 3. Database Connection Issues
```
Error: Database connection failed
```
**Solution**: Ensure PostgreSQL is running:
```bash
# Check if PostgreSQL is running
docker-compose ps

# Start if not running
docker-compose up -d postgres
```

#### 4. Dependency Conflicts
If you encounter version conflicts:
```bash
# Clean and rebuild
cargo clean
cargo build
```

### 5. Checking Service Status
```bash
# Check if service is running
ps aux | grep relay

# Check port usage
lsof -i :3002

# View service logs
RUST_LOG=debug cargo run
```

## 📝 Service Configuration

### Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `DATABASE_URL` | - | PostgreSQL connection string |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection string |
| `PORT` | `3002` | Service port |

### Config File
The service uses `config.toml` for configuration:
```toml
[server]
port = 3002
host = "0.0.0.0"

[database]
url = "postgres://postgres:postgres@localhost:5432/relay"
max_connections = 10

[redis]
url = "redis://localhost:6379"
max_connections = 10
connection_timeout_seconds = 5
```

## 🚀 Running in Production

### 1. Build Release Version
```bash
cargo build --release
```

### 2. Run as Service
```bash
# Run in background
nohup ./target/release/relay > relay.log 2>&1 &

# Or use systemd (create /etc/systemd/system/cloak-relay.service)
sudo systemctl start cloak-relay
sudo systemctl enable cloak-relay
```

### 3. Monitor the Service
```bash
# View logs
tail -f relay.log

# Check health
curl http://localhost:3002/health
```

## 📊 API Endpoints

### Available Endpoints

#### `GET /health`
- **Description**: Service health check
- **Response**: `{"status":"ok","message":"Relay service is healthy"}`

#### `GET /`
- **Description**: Service information
- **Response**: `{"service":"Cloak Relay","version":"0.1.0","status":"running"}`

#### `POST /withdraw` (Coming Soon)
- **Description**: Submit withdraw request with ZK proof
- **Status**: Will be implemented with full database/queue integration

#### `GET /status/:id` (Coming Soon)
- **Description**: Check withdraw request status
- **Status**: Will be implemented with full database/queue integration

## 🎯 Next Steps

The current service runs in **basic mode** with a simplified HTTP server. To enable full functionality:

1. **Database Integration**: Implement PostgreSQL connection and migrations
2. **Queue System**: Add Redis-based job queue for asynchronous processing
3. **Solana Integration**: Connect to Solana network for transaction submission
4. **ZK Proof Validation**: Implement SP1 proof verification
5. **Production Features**: Add rate limiting, authentication, monitoring

## 📞 Support

If you encounter issues:
1. Check the troubleshooting section above
2. Review the logs with `RUST_LOG=debug`
3. Ensure all dependencies are properly installed
4. Verify network connectivity and port availability

---

✅ **Current Status**: Basic HTTP server is working perfectly!  
🔄 **Next Phase**: Full feature implementation with database and queue integration 