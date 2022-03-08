FROM rust

COPY ./ ./
RUN cargo build
ENV DATABASE_URL=postgres://postgres:password@host.docker.internal/diesel_demo
ENV ROCKET_ADDRESS="0.0.0.0"

CMD ["cargo",  "run",  "--bin", "web"]