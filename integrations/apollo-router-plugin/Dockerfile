FROM rustlang/rust:nightly-slim as builder

ENV CARGO_TERM_COLOR always
RUN rustup component add rustfmt
RUN apt-get update && apt-get install -y npm protobuf-compiler pkg-config libssl-dev

WORKDIR /usr/src/docker-build
# create empty project for caching dependencies
RUN USER=root cargo init
COPY Cargo.toml ./
# cache dependencies
RUN cargo install --path . --locked

COPY . .
RUN touch src/main.rs
RUN cargo install --path . --locked

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
RUN update-ca-certificates
COPY --from=builder /usr/local/cargo/bin/apollo-router-plugin /bin/
COPY router.yaml /bin/
COPY supergraph-schema.graphql /bin/
CMD ["apollo-router-plugin", "-s", "/bin/supergraph-schema.graphql", "--config", "/bin/router.yaml"]
EXPOSE 4000 8088
