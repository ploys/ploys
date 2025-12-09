FROM debian:bookworm-slim

ARG BIN_DIR

COPY $BIN_DIR/ploys-api /usr/local/bin/ploys-api

EXPOSE 8080/tcp

ENTRYPOINT ["ploys-api", "serve", "--addr", "0.0.0.0:8080"]
