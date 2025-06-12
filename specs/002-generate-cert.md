# Generate TLS Certificates

I have a reverse proxy server @./src/lib.rs and a backend server @./examples/server.rs.
And the relevant config of the proxy server is @./fixtures/sample.yml
Now, I want to use tls to secure the communication between the proxy server and the backend server as well as the proxy server and the client.
In the test environment, I just use self-signed certificates with local ca.
And the ca is to certify the backend server.
Use `openssl` to generate the TLS certificates with `ECC` curve.

The domains are as follows:
- acme.com
- api.acme.com
- www.acme.com

Make sure generate cert for those domains, both for the proxy server and the backend server.
# Steps:
1. Generate key and crt for the proxy server and the backend server.
2. Generate ca to sign the certificates of the backend server.
3. All the certs are in the `./fixtures/certs` directory.
