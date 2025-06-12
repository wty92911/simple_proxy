# TLS Certificate Generation

This document describes how to generate TLS certificates for the Simple Proxy project.

## Certificate Structure

Three types of certificates are generated:

1. **CA Certificate**: Acts as the Certificate Authority that signs other certificates.
2. **Proxy Certificate**: Used by the proxy server for TLS termination.
3. **Server Certificate**: Used by the backend servers for TLS.

All certificates use the ECDH curve (prime256v1) as specified in the requirements.

## Generating Certificates

To generate all necessary certificates, run:

```bash
./scripts/generate_tls_certs.sh
```

This script will:
- Create a `certs` directory if it doesn't exist
- Generate a CA certificate
- Generate a proxy certificate signed by the CA
- Generate a server certificate signed by the CA

The certificates will be saved in the `certs` directory:

- `ca.crt` and `ca.key`: CA certificate and private key
- `proxy.crt` and `proxy.key`: Proxy certificate and private key
- `server.crt` and `server.key`: Server certificate and private key

## Using Certificates in Configuration

Update your configuration file to point to the generated certificates:

```yaml
# Global configurations for the proxy
global:
  port: 8443  # HTTPS port
  # TLS configuration for the proxy
  tls:
    cert: ./certs/proxy.crt
    key: ./certs/proxy.key
    ca: ./certs/ca.crt

# Server configurations
servers:
  - server_name:
      - example.com
    upstream: web_servers
    tls: true  # Connect to backend using TLS
```

A sample configuration file is available at `fixtures/tls_config_sample.yaml`.

## Running with TLS Support

### Start the TLS-enabled proxy

```bash
./examples/run_tls_proxy.sh
```

This script will:
1. Check if the certificates exist and generate them if needed
2. Start the proxy with TLS support using the `config_tls.yaml` configuration

### Start a TLS-enabled backend server

```bash
./examples/tls_server.sh
```

This script will:
1. Check if the certificates exist and generate them if needed
2. Start a sample server with TLS support on port 8444

## Testing TLS Setup

### Verify Certificates

You can verify the certificates using OpenSSL:

```bash
# Check CA certificate
openssl x509 -in certs/ca.crt -text -noout

# Check proxy certificate
openssl x509 -in certs/proxy.crt -text -noout

# Check server certificate
openssl x509 -in certs/server.crt -text -noout
```

### Test HTTPS Connection

You can test the HTTPS connection using curl:

```bash
# Test connection to the proxy
curl -k https://localhost:8443/health

# Test connection to the backend server directly
curl -k https://localhost:8444/health
```

The `-k` flag is required because we're using self-signed certificates. In a production environment, you would use properly trusted certificates.