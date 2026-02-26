# Bandwagon VPS Info

A lightweight, Rust-based web dashboard for monitoring multiple BandwagonHost (KiwiVM) VPS instances in one place.

## Features

- **Multi-VPS Monitoring**: View status for all your instances in a single table.
- **Usage Tracking**: Monitor monthly bandwidth consumption with visual progress bars.
- **Web Interface**: Clean, responsive HTML dashboard.
- **JSON API**: Built-in endpoint for programmatic access to your VPS data.
- **Blazing Fast**: Powered by Rust, Axum, and Tokio.

## Getting Started

### Usage

Run the server by providing your VPS credentials (VEID and API Key). You can find these in your KiwiVM control panel.

```bash
./target/release/bandwagon-vps-info --credentials "VEID1:API_KEY1,VEID2:API_KEY2"
```

#### Command Line Options

- `--credentials`: A comma-separated list of `VEID:API_KEY` pairs.
- `--listen-addr`: The address and port to bind the server to (default: `127.0.0.1:3000`).

Example:
```bash
./target/release/bandwagon-vps-info
  --credentials "123456:private_key_abc,789012:private_key_xyz"
  --listen-addr "0.0.0.0:8080"
```

## API Endpoints

- `GET /`: Displays the HTML dashboard.
- `GET /info`: Returns a JSON array containing detailed information for all configured VPS instances.

## License

[MIT](LICENSE)
