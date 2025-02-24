docker run ^
       -dit ^
       --rm ^
       --network=host ^
       --name=nodo-%1 ^
       nodos-slim ^
       new %1 127.0.0.%1