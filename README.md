# SecureTask API

API REST desenvolvida em Rust com foco em autenticação, autorização, segurança e organização de código.

O projeto foi mantido propositalmente compacto para demonstrar uma estrutura de backend próxima de um cenário real, sem adicionar camadas ou complexidades desnecessárias para uma avaliação técnica.

## Objetivo

Este repositório foi criado como uma demonstração técnica de desenvolvimento backend em Rust, abordando:

- criação de APIs REST
- autenticação e autorização
- acesso a banco de dados
- práticas de segurança
- testes automatizados
- uso de Docker
- organização de código
- integração contínua

## Funcionalidades

- cadastro de usuários
- login de usuários
- autenticação com JWT
- expiração e validação de token
- hash de senhas com Argon2id
- CRUD privado de tarefas
- PostgreSQL com migrations via SQLx
- limite de tamanho de requisição
- identificação de requisições
- logs estruturados
- encerramento seguro da aplicação
- ambiente com Docker Compose
- testes unitários
- testes de integração
- workflow de GitHub Actions com fmt, clippy e testes utilizando PostgreSQL

## Tecnologias utilizadas

- Rust 1.99+ / Edition 2024
- Axum 0.8
- Tokio
- PostgreSQL 17
- SQLx 0.9
- Argon2
- JSON Web Tokens
- Docker
- Docker Compose

---

# Como executar com Docker

## 1. Clonar o repositório

```bash
git clone https://github.com/diogo2104/securetask-rust-api.git
cd securetask-rust-api

2. Criar o arquivo de ambiente
Linux/macOS:
cp .env.example .env

Windows PowerShell:
Copy-Item .env.example .env

O arquivo .env.example já contém uma configuração adequada para ambiente local.
Antes de utilizar o projeto em um ambiente público ou de produção, altere o valor de JWT_SECRET para um segredo forte e aleatório.
3. Subir a API e o PostgreSQL
docker compose up --build

A API ficará disponível em:
http://localhost:8080

4. Verificar o funcionamento da aplicação
curl http://localhost:8080/health

Resposta esperada:
{
  "status": "ok"
}

Fluxo da API
Cadastro de usuário
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Diogo","email":"diogo@example.com","password":"RustSeguro123"}'

Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"diogo@example.com","password":"RustSeguro123"}'

A resposta contém um campo access_token.
Esse token deve ser utilizado nas rotas protegidas por meio do header:
Authorization: Bearer SEU_TOKEN

Criar uma tarefa
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Authorization: Bearer SEU_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Revisar ownership em Rust","description":"Preparar anotações para entrevista"}'

Listar tarefas
curl http://localhost:8080/api/v1/tasks \
  -H "Authorization: Bearer SEU_TOKEN"

Endpoints
Método	Endpoint	Autenticação
GET	/health	Não
POST	/api/v1/auth/register	Não
POST	/api/v1/auth/login	Não
GET	/api/v1/users/me	Sim
POST	/api/v1/tasks	Sim
GET	/api/v1/tasks	Sim
GET	/api/v1/tasks/{id}	Sim
PATCH	/api/v1/tasks/{id}	Sim
DELETE	/api/v1/tasks/{id}	Sim


Execução sem Docker
Requisitos
- Rust 1.99+
- PostgreSQL
Crie um arquivo .env com base no .env.example e ajuste a variável DATABASE_URL.
Depois execute:
cargo run

As migrations estão integradas ao projeto e são executadas automaticamente durante a inicialização da aplicação.
Testes e qualidade de código
Testes unitários
cargo test --lib

Testes completos
A suíte completa de testes requer uma instância PostgreSQL e uma variável DATABASE_URL válida.
Os testes de integração utilizam SQLx e criam um banco de dados isolado para validação do fluxo da aplicação.
cargo test --all

Verificação de formatação
cargo fmt --all -- --check

Análise com Clippy
cargo clippy --all-targets --all-features -- -D warnings

O projeto possui um workflow do GitHub Actions configurado para executar essas verificações automaticamente em pushes e pull requests.
Decisões de segurança
O projeto foi desenvolvido com algumas práticas de segurança importantes.
Senhas
As senhas nunca são armazenadas em texto puro.
O projeto utiliza Argon2id para gerar o hash das senhas.
As operações de hash e verificação são executadas com spawn_blocking, evitando bloquear as threads principais do runtime assíncrono Tokio.
JWT
Os tokens utilizam:
- HS256
- tempo de expiração
- validação de issuer
- segredo mínimo de 32 bytes
O segredo utilizado na aplicação deve ser definido por variável de ambiente.
Autorização por usuário
As tarefas pertencem ao usuário autenticado.
As consultas protegidas utilizam tanto o identificador da tarefa quanto o identificador do usuário.
Exemplo conceitual:
SELECT *
FROM tasks
WHERE id = $1
AND user_id = $2;

Isso impede que um usuário altere o UUID de uma tarefa na URL para tentar acessar dados de outro usuário.
SQL Injection
As consultas utilizam parâmetros vinculados por meio do SQLx.
Não é utilizada concatenação direta de valores recebidos pelo usuário nas instruções SQL.
Limite de requisição
O tamanho das requisições é limitado a 64 KiB.
Essa medida ajuda a reduzir o risco de requisições excessivamente grandes ou abusivas.
Tratamento de erros
Falhas de autenticação não retornam detalhes internos da aplicação.
Erros de banco de dados são registrados nos logs do servidor e convertidos em respostas genéricas para o cliente.
Variáveis de ambiente
O arquivo .env está configurado no .gitignore.
Somente o arquivo .env.example faz parte do repositório.
Isso evita o envio acidental de credenciais ou segredos reais para o GitHub.
Docker
O container da aplicação utiliza um usuário sem privilégios para executar o serviço.
Em um ambiente público, TLS, rate limiting de login e outras proteções adicionais podem ser configuradas no reverse proxy ou API Gateway utilizado na frente da aplicação.
Estrutura do projeto
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
│   │   ├── tasks.rs
│   │   └── mod.rs
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

Organização do código
A aplicação mantém a camada HTTP simples e explícita.
Os handlers são responsáveis por:
- receber as requisições
- validar os dados
- chamar as operações necessárias
- retornar as respostas HTTP
O SQLx é utilizado para comunicação com o PostgreSQL.
A autenticação é tratada por um extractor do Axum.
A estrutura foi mantida propositalmente simples para facilitar a leitura e a revisão durante uma avaliação técnica.
Mesmo sendo um projeto pequeno, existe separação entre:
- autenticação
- configuração
- handlers
- DTOs
- modelos
- rotas
- estado da aplicação
- acesso ao banco de dados
GitHub Actions
O projeto possui um workflow localizado em:
.github/workflows/ci.yml

Ele está configurado para executar verificações como:
cargo fmt --all -- --check

cargo clippy --all-targets --all-features -- -D warnings

cargo test --all

O workflow também utiliza uma instância PostgreSQL durante os testes de integração.
Exemplo de uso
O fluxo principal da aplicação é:
Usuário
   |
   v
Cadastro
   |
   v
Login
   |
   v
JWT
   |
   v
Rotas protegidas
   |
   v
Tarefas do usuário
   |
   v
PostgreSQL

Cada usuário possui acesso somente às próprias tarefas.
Health Check
A aplicação possui uma rota simples para verificar se o serviço está disponível:
GET /health

Exemplo:
curl http://localhost:8080/health

Resposta:
{
  "status": "ok"
}

Essa rota pode ser utilizada por:
- Docker
- serviços de monitoramento
- load balancers
- plataformas de hospedagem
- Kubernetes
- ferramentas de observabilidade
Logs
A aplicação utiliza logs estruturados para registrar informações sobre as requisições HTTP.
Exemplo conceitual:
INFO method=POST path=/api/v1/auth/login status=200

Informações sensíveis como senhas e tokens não devem ser registradas nos logs.
Banco de dados
O PostgreSQL é utilizado como banco principal.
As alterações da estrutura do banco são controladas por migrations.
Diretório:
migrations/

Exemplo:
001_create_users.sql
002_create_tasks.sql

Isso permite que a estrutura do banco seja reproduzida de maneira consistente em diferentes ambientes.
Docker
O projeto inclui:
Dockerfile
docker-compose.yml
.dockerignore

O Docker Compose inicia:
- aplicação Rust
- banco PostgreSQL
Com isso, quem avaliar o projeto pode iniciar o ambiente sem precisar configurar manualmente o banco.
Comando:
docker compose up --build

Para encerrar:
docker compose down

Variáveis de ambiente
As principais configurações são definidas por variáveis de ambiente.
Exemplo:
DATABASE_URL=postgres://securetask:securetask@localhost:5432/securetask
JWT_SECRET=replace-this-with-a-long-random-secret-at-least-32-bytes
JWT_TTL_MINUTES=15
PORT=8080
RUST_LOG=securetask_api=info,tower_http=info

O valor apresentado no .env.example serve apenas como referência para ambiente local.
Credenciais reais não devem ser adicionadas ao repositório.
Considerações finais
Este projeto foi desenvolvido com foco em demonstrar conhecimentos práticos de backend em Rust.
O objetivo não é criar uma aplicação excessivamente grande, mas mostrar de forma objetiva conceitos importantes como:
- desenvolvimento de APIs REST
- programação assíncrona com Tokio
- Axum
- autenticação JWT
- hash seguro de senhas
- autorização por usuário
- PostgreSQL
- SQLx
- migrations
- Docker
- testes
- GitHub Actions
- organização de código
- tratamento de erros
- segurança de APIs
A proposta é manter o código simples de revisar e, ao mesmo tempo, demonstrar preocupações reais encontradas em aplicações backend.
```
