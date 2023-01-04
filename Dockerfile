from debian:bullseye-slim as gold
run apt update && apt install -y  ca-certificates openssl libssl-dev curl

from rust:latest as chef
run cargo install cargo-chef
workdir /app

from chef as planner
copy . .
run cargo chef prepare --recipe-path recipe.json

from chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
copy . .

from builder as mistletoe-builder
workdir /app
run cargo build --release --bin mistletoe

from builder as klaxo-builder
workdir /app
run cargo build --release --bin klaxo

FROM gold AS mistletoe
WORKDIR /app
COPY --from=mistletoe-builder /app/target/release/mistletoe /usr/local/bin
ENTRYPOINT ["/usr/local/bin/mistletoe"]

FROM gold AS klaxo
WORKDIR /app
COPY --from=klaxo-builder /app/target/release/klaxo /app/
ENTRYPOINT ["/app/klaxo"]
