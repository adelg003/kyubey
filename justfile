# Install Just: https://github.com/casey/just

##########
## Rust ##
##########

# Build Debug Binary
build:
  cargo build

# Build Release Binary
build-release:
  cargo build --release

# Build and Run a Develop Binay
run:
  cargo run

# Build and Run a Release Binary
run-release:
  cargo run --release

# Check Rust Code
check:
  SQLX_OFFLINE=true cargo check --locked

# Check Rust Linting
clippy:
  SQLX_OFFLINE=true cargo clippy --locked -- --deny warnings

# Apply Rust Formating
fmt:
  cargo fmt --verbose

# Check Rust Formating
fmt-check:
  cargo fmt --check --verbose

# Install SQLx-CLI
sqlx-install:
  cargo install sqlx-cli --locked

# Check Rust Unittest
test:
  cargo test --locked

# Refresh SQLx Cache
sqlx-prepare:
  cargo sqlx prepare

# Check SQLx Cache
sqlx-check:
  cargo sqlx prepare --check

# Install Cargo Deny
deny-install:
  cargo install cargo-deny --locked

# Check Rust advisories, bans, licenses, sources
deny:
  cargo deny check


################
## PostgreSQL ##
################

# Start a local PostgreSQL instance for development.
pg-start:
  docker run \
    -dt \
    --rm \
    --name airflow_postgresql \
    --env POSTGRES_USER=airflow_user \
    --env POSTGRES_PASSWORD=password \
    --env POSTGRES_DB=airflow_db \
    --volume airflow_postgresql:/var/lib/postgresql/data \
    --publish 5432:5432 \
    docker.io/library/postgres:alpine

# Stop local PostgreSQL
pg-stop:
  docker stop airflow_postgresql

# Start local PostgreSQL via Podman
pg-start-podman:
  podman run \
    -dt \
    --rm \
    --name airflow_postgresql \
    --env POSTGRES_USER=airflow_user \
    --env POSTGRES_PASSWORD=password \
    --env POSTGRES_DB=airflow_db \
    --volume airflow_postgresql:/var/lib/postgresql/data \
    --publish 5432:5432 \
    docker.io/library/postgres:alpine

# Stop local PostgreSQL via Podman
pg-stop-podman:
  podman stop airflow_postgresql

# Connect to PostgreSQL via Rainfrog (https://github.com/achristmascarl/rainfrog)
pg-cli:
  rainfrog \
    --username=airflow_user \
    --password=password \
    --host=localhost \
    --port=5432 \
    --database=airflow_db \
    --driver=postgres


##########
## HTML ##
##########

# Appply Prettier formatting to HTML
prettier:
  npx prettier --write templates/

# Prettier Check for HTML
prettier-check:
  npx prettier --check templates/


############
## Docker ##
############

# Build the Docker image
docker-build:
  docker build . --file Containerfile --tag localhost/kyubey:latest

# Run the Docker container in Detached mode
docker-run:
  docker run \
    --name=kyubey \
    --detach \
    --rm \
    --publish=3000:3000 \
    --network=host \
    --env DATABASE_URL=postgres://airflow_user:password@localhost/airflow_db \
    --env LOG_PATH=gitignore/logs \
    localhost/kyubey:latest

# Dump logs from container
docker-logs:
 docker logs kyubey

# Follow logs from container
docker-follow:
 docker logs --follow kyubey

# Kill the Detached Docker container
docker-kill:
  docker kill kyubey

# Build the Docker image via Podman
podman-build:
  podman build . --file Containerfile --tag localhost/kyubey:latest

# Run the Docker container in Detached mode via Podman
podman-run:
  podman run \
    --name=kyubey \
    --detach \
    --rm \
    --publish=3000:3000 \
    --network=host \
    --env DATABASE_URL=postgres://airflow_user:password@localhost/airflow_db \
    --env LOG_PATH=gitignore/logs \
    localhost/kyubey:latest

# Dump logs from container via Podman
podman-logs:
 podman logs kyubey

# Follow logs from container via Podman
podman-follow:
 podman logs --follow kyubey

# Kill the Detached Docker container via Podman
podman-kill:
  podman kill kyubey


###########
## Trivy ##
###########

# Trivy Scan the code repo
trivy-repo:
  trivy repo .

# Trivy Scan the Docker image
trivy-image: docker-build
  trivy image localhost/kyubey:latest

# Trivy Scan the Docker image via podman
trivy-image-podman: podman-build
  trivy image localhost/kyubey:latest

############
## Github ##
############

# Run all Github Rust Checks
github-rust-checks: check clippy fmt-check test sqlx-check deny

# Run all Github HTML Checks
github-html-checks: prettier-check

# Run all Github Docker Checks
github-docker-checks: docker-build

# Run all Github Docker Checks via Podman
github-podman-checks: podman-build

# Run all Github Trivy Checks
github-trivy-checks: trivy-repo trivy-image

# Run all Github Trivy Checks via Podman
github-trivy-checks-podman: trivy-repo trivy-image-podman

# Run all Github Checks
github-checks: github-rust-checks github-html-checks github-docker-checks github-trivy-checks

# Run all Github Checks via Podman
github-checks-podman: github-rust-checks github-html-checks github-podman-checks github-trivy-checks-podman
