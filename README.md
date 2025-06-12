# Simple Proxy with TLS

A reverse proxy server with TLS support for secure communication.

## Certificate Generation

To generate the required TLS certificates for secure communication between:
- Proxy server and clients
- Proxy server and backend servers

Run the certificate generation script:

```bash
# Create the scripts directory if it doesn't exist
mkdir -p scripts

# Make the script executable
chmod +x scripts/generate_certs.sh

# Run the script
./scripts/generate_certs.sh
```

This script generates:
- A local Certificate Authority (CA)
- Certificates for the proxy server
- Certificates for the backend server
- All using ECC (Elliptic Curve Cryptography) for better security

## Certificate Usage

After generating certificates, update your configuration file to reference them:

```yaml
# Example configuration update
global:
  port: 3000
  tls:
    cert: ./certs/proxy/proxy.crt
    key: ./certs/proxy/proxy.key
    ca: ./certs/ca/ca.crt

# For backend servers that require TLS
upstreams:
  - name: secure_servers
    tls: true
    ca: ./certs/ca/ca.crt
    servers:
      - backend.local:3001
```

## Running the Proxy

Once certificates are generated and configuration is updated, you can run the proxy server:

```bash
cargo run -- --config ./fixtures/sample.yml
```

## Running the Example Backend Server

To run the example backend server with TLS:

```bash
cargo run --example server -- --cert ./certs/backend/backend.crt --key ./certs/backend/backend.key
```

## Features

- HTTP proxy with configurable upstream servers
- TLS support for both proxy and backend connections
- Dynamic configuration via YAML files

## Getting Started

### Generate TLS Certificates

To enable TLS support, you need to generate the required certificates:

```bash
./scripts/generate_tls_certs.sh
```

This will create the necessary CA, proxy, and server certificates using ECDH curves.

For more details, see [TLS Certificate Documentation](docs/tls_certificates.md).

### Configuration

A sample configuration is available at `fixtures/tls_config_sample.yaml`. Copy this file and modify as needed.

### Running the Proxy

#### Standard HTTP mode:

```bash
cargo run -- --config path/to/your/config.yaml
```

#### With TLS enabled:

For convenience, you can use the provided scripts to start both proxy and server with TLS enabled:

```bash
# Start the TLS-enabled proxy
./examples/run_tls_proxy.sh

# In another terminal, start a TLS-enabled backend server
./examples/tls_server.sh
```

Then test your HTTPS connection:

```bash
curl -k https://localhost:8443/health
```

## Architecture

The proxy acts as a middleman between clients and backend servers:

```
[Client] <--HTTPS--> [TLS Proxy] <--HTTPS--> [Backend Server]
```

- The proxy terminates TLS connections from clients
- It establishes new TLS connections to backend servers
- Both connections use certificates generated from the same CA

