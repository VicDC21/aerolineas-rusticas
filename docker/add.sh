#!/bin/bash

# Escribe el archivo de compose del nodo
compose_nodo() {

if [ -z $1 ]
then
       echo No se especifico un ID de nodo.
       return
fi
if [ -z $2 ]
then
       echo No se especifico la IP para el nodo de ID $1.
       return
fi
if [ -z $3 ]
then
       echo No se especifico si incluir argumentos de nodo nuevo al nodo de ID $1.
       return
fi

compose_path="./docker/compose/nodo_$1.yaml"

cat > $compose_path << EOL
services:
  nodo_$1:
    container_name: nodo-$1
    depends_on:
EOL
for nodo in ./docker/compose/nodo_*.yaml; do
    con_numeros="nodo_[0-9]+.yaml"
    nombre_nodo=$(basename $nodo)
    if [[ $nombre_nodo =~ $con_numeros ]]; then
        num_nodo=${nombre_nodo#nodo_}
        num_nodo=${num_nodo%.yaml}

        if [ $num_nodo -lt $1 ]
        then
            echo "      - nodo_$num_nodo" >> $compose_path
        fi

    fi
done
cat >> $compose_path << EOL
    extends:
        file: ./nodo_base.yaml
        service: nodo_base
    networks:
      between_nodes:
        ipv4_address: $2
    ports:
      - "127.0.0.$1:6174:6174/tcp"
      - "127.0.0.$1:8080:8080/tcp"
EOL
if [ $3 -eq 0 ]
then
       echo "    command: [\"$1\"]" >> $compose_path
else
       echo "    command: [\"new\", \"$1\", \"$2\"]" >> $compose_path
fi

# por si las moscas, que en Unix puede dar quilombos de permisos
chmod 777 $compose_path
}

# primero nos aseguramos de que los argumentos existan
if [ -z $1 ]
then
       echo No se especifico el ID del nodo
       return
fi
if [ -z $2 ]
then
       echo No se especifico la IP para el nodo de ID $1
       return
fi

# escribimos el archivo
compose_nodo $1 $2 1

# agregamos también el nodo al compose general
echo "  - ./docker/compose/nodo_$1.yaml" >> ./compose.yaml

# al CSV de IPs de nodos. Esto NO ES RELEVANTE salvo que se esté corriendo los nodos en localhost
echo "$1,$2" >> ./node_ips.csv

# al CSV de IPs de cliente
echo "$1,127.0.0.$1" >> ./client_ips.csv

# y finalmente actualizamos el compose
docker compose up --detach --no-recreate

# antes de que levanten/borren más nodos, sobreescribimos el archivo una vez más para
# hacer pensar que no es nuevo ya
compose_nodo $1 $2 0