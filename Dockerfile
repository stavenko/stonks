from debian:bullseye-slim as gold
run apt update && apt install -y  ca-certificates openssl

from rust:latest as chef
run cargo install cargo-chef
workdir app

from chef as planner
copy . .
run cargo chef prepare --recipe-path recipe.json

from chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

from builder as mistletoe-builder
workdir app
run cargo build --release -p mistletoe

from builder as klaxo-builder
workdir app
run cargo build --release -p klaxo

FROM gold AS mistletoe
WORKDIR app
COPY --from=builder /app/target/release/mistletoe /usr/local/bin
ENTRYPOINT ["/usr/local/bin/mistletoe"]

FROM gold AS klaxo
WORKDIR app
COPY --from=builder /app/target/release/klaxo /usr/local/bin
ENTRYPOINT ["/usr/local/bin/klaxo"]
