FROM rust:1.93-alpine3.22 AS chef
WORKDIR /src
RUN apk add --no-cache musl-dev protoc protobuf-dev
RUN cargo install cargo-chef --version ^0.1

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin kuberaid-agent

FROM scratch AS runtime
COPY --from=builder /src/target/release/kuberaid-agent /kuberaid-agent
ENTRYPOINT ["/kuberaid-agent"]
