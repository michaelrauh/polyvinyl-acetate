FROM rust

ARG DATABASE_URL 
ENV DATABASE_URL=$DATABASE_URL

COPY ./ ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build

ENV ROCKET_ADDRESS="0.0.0.0"

CMD ["cargo",  "run",  "--bin", "web"]
