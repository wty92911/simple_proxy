# TLS Certificates

This directory contains the TLS certificates for the proxy and backend servers.

- ca.key, ca.crt: Certificate Authority key and certificate
- proxy.key, proxy.crt: Proxy server key and certificate
- backend.key, backend.crt: Backend server key and certificate

All certificates are generated with ECC (prime256v1) and include the domains:
- acme.com
- api.acme.com
- www.acme.com
