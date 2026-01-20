# Running an ETHW (EthereumPoW) Node - Complete Guide

## Difficulty Level: **Moderate** (3/5)
Running an ETHW node is similar to running an Ethereum node before The Merge, but with less documentation and community support.

## System Requirements

### Minimum Requirements:
- **CPU**: 4+ cores (2.8GHz+)
- **RAM**: 8GB minimum (16GB recommended)
- **Storage**: 1TB SSD (NVMe preferred)
- **Network**: 25+ Mbps uncapped connection
- **OS**: Linux (CachyOS/Arch works well)

### Recommended Requirements:
- **CPU**: 8+ cores
- **RAM**: 16-32GB
- **Storage**: 2TB NVMe SSD
- **Network**: 100+ Mbps with no data caps

## Quick Start Guide for CachyOS/Arch Linux

### Option 1: Using Geth-ETHW (Recommended)

```bash
# 1. Install dependencies
sudo pacman -S base-devel git go

# 2. Clone ETHW Geth
git clone https://github.com/ethereumpow/go-ethereumpow.git
cd go-ethereumpow

# 3. Build from source
make geth

# 4. Create data directory
mkdir -p ~/ethw-node/data

# 5. Start the node
./build/bin/geth \
  --datadir ~/ethw-node/data \
  --ethw \
  --http \
  --http.addr 0.0.0.0 \
  --http.port 8545 \
  --http.api eth,net,web3,personal \
  --http.corsdomain "*" \
  --ws \
  --ws.addr 0.0.0.0 \
  --ws.port 8546 \
  --ws.api eth,net,web3 \
  --syncmode snap \
  --cache 4096
```

### Option 2: Using Docker (Easier)

```bash
# 1. Install Docker
sudo pacman -S docker docker-compose
sudo systemctl enable docker
sudo systemctl start docker
sudo usermod -aG docker $USER
# Log out and back in for group changes

# 2. Create docker-compose.yml
cat << 'EOF' > docker-compose.yml
version: '3.8'

services:
  ethw-node:
    image: ethereumpow/client-go:latest
    container_name: ethw-node
    restart: unless-stopped
    ports:
      - "8545:8545"  # HTTP RPC
      - "8546:8546"  # WebSocket RPC
      - "30303:30303" # P2P
      - "30303:30303/udp" # P2P Discovery
    volumes:
      - ./ethw-data:/root/.ethereum
    command:
      - --ethw
      - --http
      - --http.addr=0.0.0.0
      - --http.port=8545
      - --http.api=eth,net,web3,personal
      - --http.corsdomain=*
      - --ws
      - --ws.addr=0.0.0.0
      - --ws.port=8546
      - --ws.api=eth,net,web3
      - --syncmode=snap
      - --cache=4096
      - --maxpeers=50
EOF

# 3. Start the node
docker-compose up -d

# 4. Check logs
docker-compose logs -f ethw-node
```

## Sync Times

### Initial Sync Duration:
- **Snap Sync**: 12-24 hours (recommended)
- **Fast Sync**: 24-48 hours
- **Full Sync**: 3-7 days (not recommended)

### Current Blockchain Size:
- **Full Node**: ~800GB-1TB
- **Archive Node**: ~2-3TB

## Configuration Options

### Basic Configuration File
Create `~/ethw-node/config.toml`:

```toml
[Eth]
NetworkId = 10001
SyncMode = "snap"
NoPruning = false
DatabaseCache = 4096

[Node]
DataDir = "/home/r4/ethw-node/data"
IPCPath = "geth.ipc"
HTTPHost = "0.0.0.0"
HTTPPort = 8545
HTTPModules = ["eth", "net", "web3", "personal"]
WSHost = "0.0.0.0"
WSPort = 8546
WSModules = ["eth", "net", "web3"]

[Node.P2P]
MaxPeers = 50
ListenAddr = ":30303"
```

## Systemd Service (Production Setup)

Create `/etc/systemd/system/ethw-node.service`:

```ini
[Unit]
Description=ETHW Node
After=network.target

[Service]
Type=simple
User=r4
Group=r4
WorkingDirectory=/home/r4/ethw-node
ExecStart=/home/r4/go-ethereumpow/build/bin/geth \
  --datadir /home/r4/ethw-node/data \
  --ethw \
  --http \
  --http.addr 0.0.0.0 \
  --http.port 8545 \
  --http.api eth,net,web3,personal \
  --ws \
  --ws.addr 0.0.0.0 \
  --ws.port 8546 \
  --syncmode snap \
  --cache 4096 \
  --maxpeers 50
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable ethw-node
sudo systemctl start ethw-node
sudo systemctl status ethw-node
```

## Monitoring Your Node

### Check Sync Status
```bash
# Using geth console
geth attach ~/ethw-node/data/geth.ipc
> eth.syncing
> eth.blockNumber
> net.peerCount
> exit

# Using RPC
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}' \
  http://localhost:8545

# Get current block
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

### Monitor Resources
```bash
# CPU and Memory
htop

# Disk usage
df -h ~/ethw-node/data

# Network connections
ss -tunap | grep 30303

# Logs
journalctl -u ethw-node -f
```

## Firewall Configuration

```bash
# Open required ports
sudo ufw allow 30303/tcp  # P2P
sudo ufw allow 30303/udp  # Discovery
sudo ufw allow 8545/tcp   # RPC (only if external access needed)
sudo ufw allow 8546/tcp   # WebSocket (only if external access needed)
```

## Security Best Practices

1. **RPC Security**:
   ```bash
   # Use nginx reverse proxy with authentication
   sudo pacman -S nginx
   # Configure /etc/nginx/sites-available/ethw-rpc
   ```

2. **Firewall Rules**:
   - Only expose RPC ports if necessary
   - Use VPN or SSH tunnel for remote access
   - Implement rate limiting

3. **Regular Updates**:
   ```bash
   cd ~/go-ethereumpow
   git pull
   make clean
   make geth
   sudo systemctl restart ethw-node
   ```

## Connecting Your Wallet

Once synced, connect your Vaughan wallet:
```
Network Name: ETHW Local Node
RPC URL: http://localhost:8545
Chain ID: 10001
Currency Symbol: ETHW
```

## Troubleshooting

### Node Won't Sync
```bash
# Add bootnodes
--bootnodes "enode://[bootnode-address]"

# Check peers
geth attach --exec "admin.peers"

# Clear cache and resync
rm -rf ~/ethw-node/data/geth/chaindata
rm -rf ~/ethw-node/data/geth/nodes
```

### High Disk Usage
```bash
# Enable pruning
--gcmode archive  # Change to --gcmode full
--cache 2048      # Reduce cache size
```

### Connection Issues
```bash
# Check if node is listening
netstat -tulnp | grep geth

# Test RPC
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' \
  http://localhost:8545
```

## Cost Analysis

### One-Time Costs:
- **Hardware**: $0 (if using existing PC)
- **SSD Upgrade**: $50-150 (if needed)

### Ongoing Costs:
- **Electricity**: ~$5-15/month (depends on location)
- **Internet**: Part of existing plan
- **Maintenance**: 1-2 hours/month

## Pros and Cons

### Pros:
✅ Full control over your node
✅ No RPC rate limits
✅ Maximum privacy
✅ Support the network
✅ Learn blockchain technology
✅ Can serve multiple wallets

### Cons:
❌ Initial sync takes time
❌ Requires 1TB+ storage
❌ Needs stable internet
❌ Regular maintenance
❌ Electricity costs
❌ Hardware wear

## Alternative: Light Client

For easier setup with less resources:
```bash
./build/bin/geth \
  --syncmode light \
  --cache 512 \
  --maxpeers 20
```
- Syncs in minutes
- Uses <1GB storage
- Less secure/private

## Quick Decision Guide

**Run a node if:**
- You have 1TB+ free SSD space
- Stable internet connection
- Technical comfort with Linux
- Privacy is important
- Multiple ETHW transactions daily

**Use public RPC if:**
- Limited storage (<500GB)
- Metered internet
- Occasional ETHW use
- Not technically inclined
- Mobile/laptop only

## Community Resources

- ETHW Discord: [Join for support]
- GitHub: https://github.com/ethereumpow
- Documentation: https://ethereumpow.org/docs
- Block Explorer: https://www.oklink.com/ethw

## Summary

**Difficulty**: 3/5 ⭐⭐⭐
**Time to Setup**: 2-4 hours
**Time to Sync**: 12-24 hours
**Maintenance**: 1-2 hours/month
**Total Cost**: ~$5-15/month electricity

Running an ETHW node is definitely doable on your CachyOS system. The main challenges are the initial sync time and storage requirements. Once running, it provides reliable, unlimited access to the ETHW network without depending on unreliable public endpoints.