FROM ekidd/rust-musl-builder:stable AS builder

COPY . .
RUN --mount=type=cache,target=/home/rust/.cargo/git \
    --mount=type=cache,target=/home/rust/.cargo/registry \
    --mount=type=cache,sharing=private,target=/home/rust/src/target \
    sudo chown -R rust: target /home/rust/.cargo && \
    cargo build --release && \
    cp target/x86_64-unknown-linux-musl/release/polyvinyl-acetate ./polyvinyl-acetate

FROM alpine
COPY --from=builder /home/rust/src/polyvinyl-acetate .
USER 1000

ARG DATABASE_URL 
ENV DATABASE_URL=$DATABASE_URL
ENV ROCKET_ADDRESS="0.0.0.0"


CMD ["./polyvinyl-acetate"]