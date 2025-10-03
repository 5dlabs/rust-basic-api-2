# rust-basic-api

Axum-based starter API service for long-running tasks with PostgreSQL connectivity, structured logging, and containerization support.

## Getting Started

### Prerequisites

- Rust toolchain (1.80 or newer)
- Cargo package manager
- PostgreSQL instance reachable via `DATABASE_URL`

### Setup

1. Copy `.env.example` to `.env` and update the values as needed.
2. Install dependencies and run the application:

```bash
cd rust-basic-api
cargo run
```

The server listens on `0.0.0.0:3000` by default. Override the port with `SERVER_PORT` environment variable.

### Available Endpoints

| Method | Path      | Description           |
|--------|-----------|-----------------------|
| GET    | `/health` | Health check endpoint |

### Docker

Build and run the service using Docker:

```bash
docker build -t rust-basic-api .
docker run -p 3000:3000 \
  -e DATABASE_URL="postgresql://your_connection_string" \
  rust-basic-api
```

Alternatively use docker-compose:

```bash
SERVER_PORT=3000 \
DATABASE_URL=postgresql://your_connection_string \
docker compose up --build
```
