# syntax=docker/dockerfile:1
FROM --platform=$BUILDPLATFORM rustlang/rust:nightly-slim AS build
RUN rustup target add wasm32-wasi
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    cargo build --target wasm32-wasi --release

FROM scratch AS server-barebones
COPY --link --from=build /src/target/wasm32-wasi/release/wasm-sidecar.wasm wasm-sidecar.wasm
ENTRYPOINT [ "wasm-sidecar.wasm" ]
EXPOSE 8080

FROM wasmedge/slim-runtime:0.11.2 AS server-standalone
COPY --link --from=build /src/target/wasm32-wasi/release/wasm-sidecar.wasm wasm-sidecar.wasm
ENTRYPOINT [ "wasmedge", "wasm-sidecar.wasm" ]
EXPOSE 8080
