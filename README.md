# Simple Proxy

A high-performance HTTP/HTTPS reverse proxy server built with Rust and Pingora.

## Features

- HTTP/HTTPS reverse proxy with TLS support
- Dynamic upstream server configuration
- Load balancing with health checks
- Configurable TLS for both client and upstream connections
- YAML-based configuration
- Built-in caching support
- HTTP/1.1 and HTTP/2 support

## Installation

```bash
# Clone the repository
git clone https://github.com/wty92911/simple_proxy.git
cd simple_proxy

# Build the project
cargo build --release
```

## Configuration

The proxy is configured using a YAML file. Here's an example configuration:

```yaml
global:
  port: 3000
  tls:
    cert: ./certs/proxy.crt
    key: ./certs/proxy.key
    ca: ./certs/ca.crt

servers:
  - server_name: ["example.com", "www.example.com"]
    upstream: "backend_servers"
    tls: true

upstreams:
  - name: "backend_servers"
    servers:
      - "backend1:8080"
      - "backend2:8080"
```

### Configuration Options

- `global`: Global proxy settings
  - `port`: Port to listen on
  - `tls`: TLS configuration (optional)
    - `cert`: Path to certificate file
    - `key`: Path to private key file
    - `ca`: Path to CA certificate file (optional)

- `servers`: List of server configurations
  - `server_name`: List of hostnames to match
  - `upstream`: Name of the upstream server group
  - `tls`: Whether to use TLS for upstream connections

- `upstreams`: List of upstream server groups
  - `name`: Unique name for the upstream group
  - `servers`: List of backend server addresses

## Usage

### Running the Proxy

```bash
# Run with default configuration
cargo run -- --config ./fixtures/sample.yml

# Run with custom configuration
cargo run -- --config /path/to/your/config.yml
```

### Example Backend Server

The repository includes an example backend server that can be used for testing:

```bash
# Run the example server
cargo run --example server -- --port 3001 --tls --cert ./certs/backend.crt --key ./certs/backend.key
```

## Development

### Prerequisites

- Rust 1.70 or later
- OpenSSL development libraries

### Building

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test
```

### Project Structure

```
src/
├── conf/           # Configuration handling
├── proxy/          # Proxy implementation
│   ├── route.rs    # Routing logic
│   └── utils.rs    # Utility functions
└── main.rs         # Application entry point
```

## License

MIT License - see LICENSE file for details

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

