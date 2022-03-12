FROM ekidd/rust-musl-builder:latest AS builder

ADD --chown=rust:rust . ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/polyvinyl-acetate \
    /usr/local/bin/

ARG DATABASE_URL 
ENV DATABASE_URL=$DATABASE_URL
ENV ROCKET_ADDRESS="0.0.0.0"

CMD /usr/local/bin/polyvinyl-acetate