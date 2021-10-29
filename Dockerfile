FROM rust:1.56-slim-bullseye as rust
WORKDIR /src
RUN apt-get update && apt-get install -y git pkg-config libssl-dev
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY Cargo.toml .
RUN sed -i '/.*build.rs.*/d' Cargo.toml
COPY Cargo.lock .
RUN cargo build --release
COPY . /src
RUN cargo build --release

FROM debian:bullseye-slim
#RUN useradd -ms /bin/bash -u 1000 pages
#RUN mkdir -p /var/www/pages && chown pages /var/www/pages
RUN apt-get update && apt-get install -y ca-certificates
COPY scripts/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x  /usr/local/bin/entrypoint.sh
COPY --from=rust /src/target/release/pages /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
