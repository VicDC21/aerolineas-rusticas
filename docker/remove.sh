#!/bin/bash

cargo run \
      --package=server \
      --bin=nd \
      delete $1

echo Esperando a que se relocalicen los nodos...
sleep 10s

docker container rm -f nodo-$1