#######################################################################
################  -= MANIFIESTO DE COPIA COMPLETA =-   ################
#######################################################################
### Esto genera una copia tal cual del directorio, con los archivos ###
### fuentes incluidos, para su uso en depuración o similar.         ###
#######################################################################

ARG RUST_VERSION=1.84

FROM rust:${RUST_VERSION}

WORKDIR /aerolineas_rusticas

COPY . .

# Cliente-Nodo
EXPOSE 8080/tcp
# Entre nodos
EXPOSE 6174/tcp