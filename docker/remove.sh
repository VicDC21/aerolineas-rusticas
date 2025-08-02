#!/bin/bash

# Lee de un archivo y borra una línea dada.
# Acepta una ruta como 1er arg y un string que es la línea a borrar como 2do arg
borrar_linea() {
      if [ -z $1 ]
      then
            echo No se especifico una ruta. No se hace nada...
            return
      fi
      if [[ -z $2 ]]
      then
            echo No se especifico una linea que eliminar en "$1". No se hace nada...
            return
      fi

      sed -i "/$2/d" $1
}

# Lee de un archivo (se asume csv) y borra una línea dada si es que tiene coincidencia con el 2do arg pasado.
# Acepta una ruta como 1er arg y un string que es el numero de id de un nodo como 2do arg.
borrar_linea_segun_id() {
    if [ -z "$1" ]; then
        echo "No se especificó un ID de nodo. No se hace nada..."
        return
    fi

    # Borra la línea que comienza con el ID seguido de una coma (ej: 10,)
    sed -i "/^$2,/d" "$1"
}


if [ -z $1 ]
then
      echo No se incluyo un ID de nodo.
      return
fi

# borramos el compose del nodo
rm -f ./docker/compose/nodo_$1.yaml

# borramos la IP del CSV del cliente
borrar_linea "./client_ips.csv" "$1,127.0.0.$1"

# borramos la IP del CSV de los nodos
borrar_linea_segun_id "./node_ips.csv" "$1"

# y lo sacamos del compose general
borrar_linea "./compose.yaml" "  - .\/docker\/compose\/nodo_$1.yaml"

# mandamos comando para parar el nodo
cargo run \
      --package=server \
      --bin=nd \
      delete $1

# un poco de tiempo para que se acostumbren los nodos
echo Esperando a que se relocalicen los nodos...
sleep 10s

# y finalmente actualizamos los nodos
docker compose up --detach --no-recreate --remove-orphans