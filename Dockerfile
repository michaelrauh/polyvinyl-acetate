FROM rust

COPY ./ ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build

ENV DATABASE_URL=postgres://postgres:password@postgres-k-postgresql.default.svc.cluster.local/postgres 

ENV ROCKET_ADDRESS="0.0.0.0"

CMD ["cargo",  "run",  "--bin", "web"]
