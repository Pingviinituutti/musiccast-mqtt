FROM gcr.io/distroless/static@sha256:7198a357ff3a8ef750b041324873960cf2153c11cc50abb9d8d5f8bb089f6b4e
COPY target/x86_64-unknown-linux-musl/release/musiccast-mqtt /usr/local/bin/musiccast-mqtt
WORKDIR /app
CMD ["musiccast-mqtt"]
