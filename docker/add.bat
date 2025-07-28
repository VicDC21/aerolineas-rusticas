@echo off

@REM para poder usar variables con !esto!
setlocal EnableDelayedExpansion
GOTO:cuerpo

@REM Escribe el archivo de compose del nodo
:compose_nodo
IF "%~1"=="" (
    echo No se especifico un ID de nodo.
    GOTO:EOF
)
IF "%~2"=="" (
    echo No se especifico la IP para el nodo de ID %~1.
    GOTO:EOF
)
IF "%~3"=="" (
    echo No se especifico si incluir argumentos de nodo nuevo al nodo de ID %~1.
    GOTO:EOF
)

(
echo services:
echo   nodo_%~1:
echo     container_name: nodo-%~1
echo     depends_on:
for %%I in (./docker/compose/nodo_*.yaml) do (
    SET nodo=%%I
    SET nodo=!nodo:~5,-5!
    SET /A nodo=nodo
    IF !nodo! NEQ 0 IF !nodo! LSS %~1 (
echo       - nodo_!nodo!
    )
)
echo     extends:
echo         file: ./nodo_base.yaml
echo         service: nodo_base
echo     networks:
echo       between_nodes:
echo         ipv4_address: %~2
echo     ports:
echo       - "127.0.0.%~1:6174:6174/tcp"
echo       - "127.0.0.%~1:8080:8080/tcp"
IF %~3==0 (
echo     command: ["%~1"]
) ELSE (
echo     command: ["new", "%~1", "%~2"] 
)
) > ./docker/compose/nodo_%~1.yaml

GOTO:EOF


@REM el resto del script
:cuerpo

@REM primero nos aseguramos de que los argumentos existan
IF "%1"=="" (
    echo No se especifico el ID del nodo.
    GOTO:EOF
)
IF "%2"=="" (
    echo No se especifico la IP para el nodo de ID %1.
    GOTO:EOF
)

@REM escribimos el archivo
CALL:compose_nodo %1 %2 1

@REM agregamos también el nodo al compose general
(
echo   - ./docker/compose/nodo_%1.yaml
) >> ./compose.yaml

@REM al CSV de IPs de cliente
(
echo %1,127.0.0.%1
) >> ./client_ips.csv

@REM y finalmente actualizamos el compose
docker compose up --detach --no-recreate

@REM antes de que levanten/borren más nodos, sobreescribimos el archivo una vez más para
@REM hacer pensar que no es nuevo ya
CALL:compose_nodo %1 %2 0