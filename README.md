# SecureTask API

A small REST API written in Rust with a focus on authentication, authorization, secure defaults and maintainable code.

The project is intentionally compact: it demonstrates a production-oriented backend structure without adding unnecessary layers that do not help a technical evaluation.

## Purpose

This repository was created as a compact technical demonstration of backend development in Rust, focusing on API design, authentication, authorization, database access, security practices, testing and containerized execution.

## Features

- User registration and login
- JWT authentication with expiration and issuer validation
- Password hashing with Argon2id
- Private task CRUD
- PostgreSQL with SQLx migrations
- Request size limit
- Request IDs and structured HTTP tracing
- Graceful shutdown
- Docker Compose environment
- Unit and integration tests
- GitHub Actions workflow with `fmt`, `clippy` and PostgreSQL-backed tests

## Stack

- Rust 1.99+ / Edition 2024
- Axum 0.8
- Tokio
- PostgreSQL 17
- SQLx 0.9
- Argon2
- JSON Web Tokens
- Docker / Docker Compose

---

## Quick start with Docker

### 1. Clone the repository

```bash
git clone https://github.com/diogo2104/securetask-rust-api.git
cd securetask-rust-api
