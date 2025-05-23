#################
## Build Image ##
#################
FROM rust:alpine as builder

# Setup dependencies
RUN apk add --no-cache musl-dev npm

# Copy files to build Rust Application
WORKDIR /opt/kyubey
COPY ./build.rs ./build.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./package.json ./package.json
COPY ./package-lock.json ./package-lock.json
COPY ./.sqlx ./.sqlx
COPY ./src ./src
COPY ./tailwind.css ./tailwind.css

# Build Rust Application
RUN SQLX_OFFLINE=true cargo build --release --locked


###################
## Runtime Image ##
###################
FROM alpine:3

# Setup dependencies
RUN apk add --no-cache alpine-conf curl

# Setup Catalog2 User
RUN setup-user kyubey
USER kyubey
WORKDIR /home/kyubey

# Copy over complied runtime binary
COPY --from=builder /opt/kyubey/target/release/kyubey /usr/local/bin/kyubey

# Setup Healthcheck
HEALTHCHECK CMD curl --fail http://localhost:3000

# Run Catalog2
ENV RUST_BACKTRACE=full
ENTRYPOINT ["kyubey"]
