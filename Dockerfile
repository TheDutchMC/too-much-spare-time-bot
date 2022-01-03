# Program builder
FROM docker-registry.k8s.array21.dev/rust-base:latest as BUILDER
COPY ./src /usr/src/src/
COPY ./Cargo.toml /usr/src/src
COPY ./migrations /usr/src/migrations/

WORKDIR /usr/src/src/

ENV RUSTFLAGS='-C link-arg=-s'
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime image
FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=BUILDER /usr/src/src/target/x86_64-unknown-linux-musl/release/too-much-spare-time /usr/local/bin/too-much-spare-time

RUN chmod a+rx /usr/local/bin/*
RUN adduser too-much-spare-time -s /bin/false -D -H
USER too-much-spare-time

WORKDIR /usr/local/bin
ENTRYPOINT [ "/usr/local/bin/too-much-spare-time", "--config", "/app/config.yaml" ]