FROM rust:1.99.1-slim-trixie AS build
WORKDIR /app
ENV SQLX_OFFLINE=true
COPY . .
RUN cargo build --release --locked --bin api --bin worker

FROM debian:trixie-20261020-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
RUN useradd --system --no-create-home clinicore
COPY --from=build /app/target/release/api /app/target/release/worker /usr/local/bin/
USER clinicore
EXPOSE 3333
CMD ["api"]
