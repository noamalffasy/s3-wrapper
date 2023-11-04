FROM rust:1.67 as builder

WORKDIR /usr/src/s3-wrapper
COPY . .

RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/s3-wrapper /usr/local/bin/s3-wrapper

CMD ["s3-wrapper"]