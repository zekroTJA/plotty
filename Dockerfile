FROM rust:alpine AS build
WORKDIR /build
COPY plotty plotty
COPY minecraft-uuid minecraft-uuid
COPY minecraft-uuid-cli minecraft-uuid-cli
COPY Cargo.lock .
COPY Cargo.toml .
RUN apk add musl-dev
RUN cargo build -p plotty --release

FROM alpine:latest AS release
COPY --from=build /build/target/release/plotty /bin/plotty
ENTRYPOINT [ "/bin/plotty" ]