FROM rust:alpine AS build
WORKDIR /build
COPY src src
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release

FROM alpine:latest AS release
COPY --from=build /build/target/release/plotty /bin/plotty
ENTRYPOINT [ "/bin/plotty" ]