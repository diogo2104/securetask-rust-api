# SecureTask API

A small REST API written in Rust with a focus on authentication, authorization, secure defaults and maintainable code.

The project is intentionally compact: it demonstrates a production-oriented backend structure without adding layers that do not help a technical evaluation.

## Features

- user registration and login
- JWT authentication with expiration and issuer validation
- password hashing with Argon2id
- private task CRUD
- PostgreSQL with SQLx migrations
- request size limit
- request IDs and structured HTTP tracing
- graceful shutdown
- Docker Compose environment
- unit and integration tests
- GitHub Actions with `fmt`, `clippy` and PostgreSQL-backed tests

## Stack

- Rust 1.99 / Edition 2024
- Axum 0.8
- Tokio
- PostgreSQL 17
- SQLx 0.9
- Argon2
- JSON Web Tokens
- Docker / Docker Compose

## Quick start with Docker

### 1. Clone the repository

```bash
git clone <repository-url>
cd securetask-rust-api
```

### 2. Create the environment file

Linux/macOS:

```bash
cp .env.example .env
```

Windows PowerShell:

```powershell
Copy-Item .env.example .env
```

The example file is ready for local development. Before using the project outside a local environment, replace `JWT_SECRET` with a strong random secret.

### 3. Start the API and PostgreSQL

```bash
docker compose up --build
```

The API will be available at:

```text
http://localhost:8080
```

### 4. Check the service

```bash
curl http://localhost:8080/health
```

Expected response:

```json
{
  "status": "ok"
}
```

## API flow

### Register

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Diogo","email":"diogo@example.com","password":"RustSeguro123"}'
```

### Login

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"diogo@example.com","password":"RustSeguro123"}'
```

The response contains an `access_token`. Use it as a Bearer token in protected routes.

### Create a task

```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Review Rust ownership","description":"Prepare notes for interview"}'
```

### List tasks

```bash
curl http://localhost:8080/api/v1/tasks \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## Endpoints

| Method | Endpoint | Authentication |
|---|---|---|
| GET | `/health` | No |
| POST | `/api/v1/auth/register` | No |
| POST | `/api/v1/auth/login` | No |
| GET | `/api/v1/users/me` | Yes |
| POST | `/api/v1/tasks` | Yes |
| GET | `/api/v1/tasks` | Yes |
| GET | `/api/v1/tasks/{id}` | Yes |
| PATCH | `/api/v1/tasks/{id}` | Yes |
| DELETE | `/api/v1/tasks/{id}` | Yes |

## Run without Docker

Requirements:

- Rust 1.99+
- PostgreSQL

Create `.env` from `.env.example`, update `DATABASE_URL`, then run:

```bash
cargo run
```

Migrations are embedded in the binary and executed automatically during startup.

## Tests and code quality

Unit tests:

```bash
cargo test --lib
```

Full test suite requires a PostgreSQL instance and a valid `DATABASE_URL` because the integration test creates an isolated test database using SQLx:

```bash
cargo test --all
```

Static checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

The same checks run automatically in GitHub Actions on pushes and pull requests.

## Security decisions

- passwords are hashed with Argon2id and never stored in plain text
- password hashing and verification run with `spawn_blocking` to avoid blocking Tokio worker threads
- JWTs use HS256, expiration validation and a fixed issuer
- the JWT secret must contain at least 32 bytes
- protected task queries always include the authenticated `user_id`
- a user cannot access another user's tasks by changing a task UUID
- SQL values are passed using SQLx bind parameters
- request bodies are limited to 64 KiB
- authentication failures do not expose internal details
- database errors are logged server-side and returned as generic API errors
- `.env` is excluded from version control
- the Docker runtime uses an unprivileged user

For a public deployment, TLS and login rate limiting should normally be enforced by the reverse proxy/API gateway in front of this service.

## Project structure

```text
securetask-rust-api/
├── .github/
│   └── workflows/
│       └── ci.yml
├── migrations/
│   ├── 001_create_users.sql
│   └── 002_create_tasks.sql
├── src/
│   ├── handlers/
│   │   ├── auth.rs
│   │   ├── health.rs
│   │   └── tasks.rs
│   ├── auth.rs
│   ├── config.rs
│   ├── dto.rs
│   ├── error.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── models.rs
│   ├── routes.rs
│   └── state.rs
├── tests/
│   └── api_flow.rs
├── .dockerignore
├── .editorconfig
├── .env.example
├── .gitignore
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
└── rust-toolchain.toml
```

## Design notes

The API keeps the HTTP layer small and explicit. Handlers validate input, SQLx performs parameterized database access, and authentication is handled by an Axum request extractor.

For a project of this size, this structure keeps the code easy to review while still separating authentication, configuration, HTTP handlers, DTOs, models and application state.
