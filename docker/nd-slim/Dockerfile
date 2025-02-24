#######################################################################
################  -= MANIFIESTO DE NODOS MINIMAL =-   #################
#######################################################################
### Esto genera una imagen para usar como plantilla en contenedores ###
### de nodos.                                                       ###
###                                                                 ###
### Esta versión contiene nada salvo el binario mismo y los datos   ###
### estáticos necesarios para funcionar.                            ###
#######################################################################

ARG RUST_VERSION=1.84

FROM rust:${RUST_VERSION}-slim AS builder

WORKDIR /aerolineas_rusticas

COPY . .

RUN apt-get update
RUN apt-get install musl musl-dev musl-tools

# Este target incluye librerías de C en MUSL por lo que el binario
# resulta completamente estático.
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build \
    --target=x86_64-unknown-linux-musl \
    --package server \
    --release

FROM scratch AS final

COPY --from=builder /aerolineas_rusticas/target/x86_64-unknown-linux-musl/release/nd /bin/nd
COPY ./datasets /datasets
COPY ./scripts /scripts
COPY cert.pem custom.csr custom.key node_ips.csv users.csv ./

# Cliente-Nodo
EXPOSE 8080/tcp
# Entre nodos
EXPOSE 6174/tcp

ENTRYPOINT ["/bin/nd"]