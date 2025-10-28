# Quick Start: Reboot-Resilient CAN Networking

## What Changed?

✅ Containers with CAN networking now **automatically recover after system reboot**  
✅ No manual intervention required  
✅ 100% backward compatible  

## How It Works

The plugin now saves network configurations to a persistent JSON file:
- **Location:** `/var/lib/docker/network/files/rustycan4docker-networks.json`
- **When:** Automatically on network creation
- **Why:** Survives reboots, doesn't depend on Docker API

## Testing Your Setup

### 1. Start a Container

```bash
cd /home/nomadic/src/rustycan4docker
docker compose -f docker-compose.simple.yml up -d
```

### 2. Verify CAN Interface

```bash
docker exec can-test-simple ip link show can0
# Should show: can0@ifXX: <NOARP,UP,LOWER_UP,M-DOWN>
```

### 3. Simulate Reboot

```bash
sudo systemctl restart docker
sleep 15
```

### 4. Verify Recovery

```bash
docker compose -f docker-compose.simple.yml ps
# Container should show "Up"

docker exec can-test-simple ip link show can0  
# CAN interface should still exist
```

## Build & Install

```bash
# Build release binary
cargo build --release

# Build Docker plugin
cd docker-plugin
./build-plugin.sh

# Enable plugin
docker plugin enable nomadicdrones/rustycan4docker:latest
```

## Example docker-compose.yml

```yaml
version: '3.8'

services:
  my-can-app:
    image: my-app:latest
    networks:
      - canbus0
    restart: unless-stopped

networks:
  canbus0:
    driver: nomadicdrones/rustycan4docker:latest
    driver_opts:
      vxcan.dev: can
      vxcan.peer: can
      vxcan.id: 0
```

## Troubleshooting

### Check Plugin Status
```bash
docker plugin ls
# Should show: ENABLED = true
```

### View Persisted Networks
```bash
docker plugin inspect nomadicdrones/rustycan4docker:latest --format '{{.ID}}' | \
  xargs -I {} sudo cat /var/lib/docker/plugins/{}/rootfs/var/lib/docker/network/files/rustycan4docker-networks.json
```

### Check Recovery Logs
```bash
journalctl -u docker -f | grep -E "Loaded.*network configurations|Successfully recovered"
```

### Common Issues

**Container doesn't start after reboot:**
```bash
# Check if network config was persisted
# Should see JSON file with your network ID
```

**CAN interface missing:**
```bash
# Check plugin logs
journalctl -u docker | grep "rustycan4docker"
```

## Success Indicators

✅ Container shows "Up" status after Docker restart  
✅ CAN interface (`can0`) exists in container  
✅ Log shows "Loaded X network configurations from file"  
✅ No "failed to attach endpoint" errors  

## That's It!

The plugin is now production-ready and handles reboots automatically. No configuration changes needed - just deploy and enjoy automatic recovery!

