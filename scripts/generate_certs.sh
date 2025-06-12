#!/bin/bash
set -e

# Create certs directory if it doesn't exist
mkdir -p fixtures/certs

# Generate CA key and certificate
echo "Generating CA key and certificate..."
openssl ecparam -name prime256v1 -genkey -noout -out fixtures/certs/ca.key
openssl req -new -x509 -key fixtures/certs/ca.key -out fixtures/certs/ca.crt -days 365 \
  -subj "/C=US/ST=State/L=City/O=Organization/OU=IT/CN=acme-ca"

# Generate proxy server key and CSR
echo "Generating proxy server key and certificate..."
openssl ecparam -name prime256v1 -genkey -noout -out fixtures/certs/proxy.key
openssl req -new -key fixtures/certs/proxy.key -out fixtures/certs/proxy.csr \
  -subj "/C=US/ST=State/L=City/O=Organization/OU=IT/CN=acme.com"

# Create config file for SAN
cat > fixtures/certs/proxy-san.cnf <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = acme.com

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = acme.com
DNS.2 = api.acme.com
DNS.3 = www.acme.com
EOF

# Sign the proxy certificate with our own CA
openssl x509 -req -in fixtures/certs/proxy.csr -CA fixtures/certs/ca.crt -CAkey fixtures/certs/ca.key \
  -CAcreateserial -out fixtures/certs/proxy.crt -days 365 -extensions v3_req -extfile fixtures/certs/proxy-san.cnf

# Generate backend server key and CSR
echo "Generating backend server key and certificate..."
openssl ecparam -name prime256v1 -genkey -noout -out fixtures/certs/backend.key
openssl req -new -key fixtures/certs/backend.key -out fixtures/certs/backend.csr \
  -subj "/C=US/ST=State/L=City/O=Organization/OU=IT/CN=acme.com"

# Create config file for backend SAN
cat > fixtures/certs/backend-san.cnf <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = acme.com

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = acme.com
DNS.2 = api.acme.com
DNS.3 = www.acme.com
EOF

# Sign the backend certificate with our own CA
openssl x509 -req -in fixtures/certs/backend.csr -CA fixtures/certs/ca.crt -CAkey fixtures/certs/ca.key \
  -CAcreateserial -out fixtures/certs/backend.crt -days 365 -extensions v3_req -extfile fixtures/certs/backend-san.cnf

# Create README in the certs directory
cat > fixtures/certs/README.md <<EOF
# TLS Certificates

This directory contains the TLS certificates for the proxy and backend servers.

- ca.key, ca.crt: Certificate Authority key and certificate
- proxy.key, proxy.crt: Proxy server key and certificate
- backend.key, backend.crt: Backend server key and certificate

All certificates are generated with ECC (prime256v1) and include the domains:
- acme.com
- api.acme.com
- www.acme.com
EOF

# Clean up CSR files
rm fixtures/certs/proxy.csr fixtures/certs/backend.csr
rm fixtures/certs/proxy-san.cnf fixtures/certs/backend-san.cnf

echo "Certificate generation complete!"
echo "Certificates are stored in fixtures/certs/"
ls -la fixtures/certs/