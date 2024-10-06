# README.md

# Aplicativo em Rust, Docker, PostgreSQL e Svelte

## Introdução

Este é um aplicativo desenvolvido em Rust, utilizando Docker para containerização, PostgreSQL como banco de dados e Svelte para o frontend.

## Requisitos

* Docker latest
* Docker Compose latest
* Rust v1.78
* PostgreSQL v13

## Iniciando o Projeto

### Backend

Para iniciar o container do backend, execute o comando abaixo:

```bash
docker compose up -d rustapp
```

### Frontend

Para iniciar o frontend, execute o comando abaixo:

```bash
npm run dev
```

### Banco de Dados

O banco de dados utilizado é o PostgreSQL. Ao iniciar o backend, o banco de dados é iniciado automaticamente. Para criar o banco de dados separadamente, execute o comando abaixo:

```bash
docker compose up -d postgres
```

## Configuração

### Variáveis de Ambiente

As variáveis de ambiente estão definidas no arquivo `.env`. É necessário criar um arquivo `.env` na raiz do diretório `backend` com as seguintes variáveis:

```makefile
POSTGRES_USER="user"
POSTGRES_PASSWORD="password"
POSTGRES_DB="database-name"
```

### Docker Compose

O arquivo `docker-compose.yml` é utilizado para definir os serviços do Docker. Para executar o container do backend, é necessário executar o comando abaixo:

```bash
docker compose up -d rustapp
```

## Executando o Aplicativo

Para executar o aplicativo, é necessário iniciar o container do backend e o frontend. Execute os comandos abaixo:

```bash
cd backend && docker compose up -d rustapp
cd ..
cd frontend && npm run dev
```

## Contribuição

Contribuições são bem-vindas! Para contribuir, é necessário criar um fork do repositório e enviar um pull request com as alterações.

## Licença

Este projeto é licenciado sob a licença MIT.