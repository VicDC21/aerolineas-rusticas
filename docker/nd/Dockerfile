#######################################################################
#####################  -= MANIFIESTO DE NODOS=-   #####################
#######################################################################
### Esto genera una imagen para usar como plantilla en contenedores ###
### de nodos.                                                       ###
#######################################################################

ARG RUST_VERSION=1.84

FROM rust:${RUST_VERSION}-slim

WORKDIR /aerolineas_rusticas

COPY . .

# Compilar archivos fuentes y todas sus dependencias, pero sólo las que interesan
# a 'server'
RUN ["cargo", "build", "--package", "server"]

# Cliente-Nodo
EXPOSE 8080/tcp
# Entre nodos
EXPOSE 6174/tcp

# Esto se asegura de que la instancia de cargo sea el proceso mismo, y no un subproceso de shell;
# como sí ocurriría si lo invocamos desde consola.
ENTRYPOINT ["cargo", "run", "--package", "server", "--bin", "nd"]