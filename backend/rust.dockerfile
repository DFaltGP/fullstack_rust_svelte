# Build stage
# Image with rustup and Cargo already installed
FROM rust:1.78-bullseye

# Workdir
WORKDIR /app

# Copying necessary files
COPY src /app/src
COPY migrations /app/migrations
COPY Cargo.toml /app/Cargo.toml

# Defining an environment variable [database]://[user:[password]]@host(container_name):[port]/[database_name]
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

# Compiling source code
RUN cargo build --release --verbose

# Exposing port for app connection
EXPOSE 8080

# Command to execute the app
CMD [ "/app/target/release/backend" ]