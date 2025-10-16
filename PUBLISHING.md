# Publishing the Docker Plugin

This document describes how to build and publish the RustyCAN4Docker plugin to Docker Hub.

## Prerequisites

1. **Docker Hub Account**: You need a Docker Hub account to publish plugins
2. **GitHub Secrets**: Configure the following secrets in your GitHub repository:
   - `DOCKER_USERNAME`: Your Docker Hub username
   - `DOCKER_TOKEN`: A Docker Hub access token (create at [Docker Hub Security Settings](https://hub.docker.com/settings/security))

## Local Development

### Building the Plugin Locally

```bash
cd docker-plugin
sudo ./build-plugin.sh
```

This will:
1. Build the Docker image containing your Rust application
2. Create a Docker plugin from the image
3. Install the plugin locally for testing

### Testing the Plugin

```bash
# List installed plugins
docker plugin ls

# Test the plugin (example)
docker network create -d nomadicdrones/rustycan4docker my-can-network
```

## Publishing Process

### Automatic Publishing (Recommended)

The plugin is automatically built and published for both AMD64 and ARM64 architectures when you create a release:

1. **Create a Release on GitHub**:
   ```bash
   # Tag your commit
   git tag v0.1.0
   git push origin v0.1.0
   
   # Or create a release through GitHub UI
   ```

2. **GitHub Actions will automatically**:
   - Build the plugin for both AMD64 and ARM64 architectures
   - Test both builds
   - Push to Docker Hub as:
     - `nomadicdrones/rustycan4docker-amd64:v0.1.0`
     - `nomadicdrones/rustycan4docker-arm64:v0.1.0`
     - `nomadicdrones/rustycan4docker:v0.1.0` (multi-arch manifest)
   - Also tag as `latest` for releases with multi-arch support

### Architecture-Specific Publishing

The build process creates both architecture-specific plugins and a unified multi-architecture manifest:

- **AMD64**: `nomadicdrones/rustycan4docker-amd64:latest`
- **ARM64**: `nomadicdrones/rustycan4docker-arm64:latest`  
- **Multi-arch**: `nomadicdrones/rustycan4docker:latest` (automatically selects correct architecture)

### Manual Publishing

If you need to publish manually for a specific architecture:

```bash
# Build for current architecture
cd docker-plugin
export PLUGIN_NAME="nomadicdrones/rustycan4docker-$(uname -m):v0.1.0"
sudo ./build-plugin.sh

# Login to Docker Hub
docker login

# Push the plugin
docker plugin push "nomadicdrones/rustycan4docker-$(uname -m):v0.1.0"
```

## Installation for Users

Once published, users can install your plugin with automatic architecture detection:

```bash
# Install the plugin (automatically selects correct architecture)
docker plugin install nomadicdrones/rustycan4docker:latest

# Or install a specific version
docker plugin install nomadicdrones/rustycan4docker:v0.1.0

# Enable the plugin
docker plugin enable nomadicdrones/rustycan4docker
```

### Architecture-Specific Installation

If you need to install a specific architecture:

```bash
# For AMD64 systems
docker plugin install nomadicdrones/rustycan4docker-amd64:latest

# For ARM64 systems  
docker plugin install nomadicdrones/rustycan4docker-arm64:latest

# Enable the plugin
docker plugin enable nomadicdrones/rustycan4docker-amd64  # or -arm64
```

### Using the Plugin

```bash
# Use the plugin (same regardless of architecture)
docker network create -d nomadicdrones/rustycan4docker my-can-network
```

## Versioning

- Version numbers are extracted from `Cargo.toml`
- Git tags should follow semantic versioning (e.g., `v0.1.0`, `v1.2.3`)
- Each release creates both a versioned tag and updates `latest`

## Troubleshooting

### Build Issues
- Ensure Docker is running and you have sudo privileges
- Check that the Rust code compiles: `cargo build --release`

### Publishing Issues
- Verify Docker Hub credentials are correct
- Check that the plugin name follows Docker Hub naming conventions
- Ensure you have push permissions to the Docker Hub repository

### Plugin Installation Issues
- Make sure the plugin is correctly formatted
- Check that required capabilities and network settings are proper
- Verify the plugin socket and interface configuration