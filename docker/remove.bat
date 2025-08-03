@echo off

setlocal EnableDelayedExpansion
GOTO:cuerpo

@REM Lee de un archivo y borra una línea dada.
@REM Acepta una ruta como 1er arg y un string que es la línea a borrar como 2do arg
:borrar_linea
    IF "%~1"=="" (
        echo No se especifico una ruta. No se hace nada...
        GOTO:EOF
    )
    IF "%~2"=="" (
        echo No se especifico una linea que eliminar en "%~1". No se hace nada...
        GOTO:EOF
    )
    @REM Si %~3 NO es '0', entonces %~2 pasa a ser un substring con el que la línea debería empezar

    (
        FOR /F "tokens=* delims=" %%l in (%~1) DO (
            IF "%~3" NEQ "0" (
                SET starts=%%l
                SET starts=!starts:~0,2!
                IF "!starts!" NEQ "%~2" (
                    echo %%l
                )
            ) ELSE (
                IF "%%l" NEQ "%~2" (
                    echo %%l
                )
            )
        )
    ) > %~1_temp

    @REM reemplazamos '/' por '\' de ser necesario
    SET ruta_win=%~1
    SET ruta_win=%ruta_win:/=\%
    DEL %ruta_win%

    REN "%ruta_win%_temp" %~nx1

@REM aunque diga EOF, en el contexto de una CALL esto vuelve a donde llamaron a la funcion
GOTO:EOF

@REM el resto del script, después de las funciones
:cuerpo

IF "%1"=="" (
      echo No se incluyo un ID de nodo.
      GOTO:EOF
)

@REM borramos el compose del nodo
@REM necesariamente con backslashes
DEL ".\docker\compose\nodo_%1.yaml"

@REM borramos el nodo de todo YAML de nodo con número más alto
for %%I in (./docker/compose/nodo_*.yaml) do (
    SET nodo=%%I
    SET nodo=!nodo:~5,-5!
    SET /A nodo=nodo
    IF !nodo! NEQ 0 IF !nodo! GTR %1 (
        @REM hay que reconstruir el string de nuevo, ya que %%I fue mangleado
        CALL:borrar_linea "./docker/compose/nodo_!nodo!.yaml" "      - nodo_%1"
    ) 
)

@REM borramos la IP del CSV del cliente
CALL:borrar_linea "./client_ips.csv" "%1,127.0.0.%1"

@REM y del CSV de nodos
CALL:borrar_linea "./node_ips.csv" %1 1

@REM y lo sacamos del compose general
CALL:borrar_linea "./compose.yaml" "  - ./docker/compose/nodo_%1.yaml"

@REM mandamos comando para parar el nodo
cargo run ^
      --package=server ^
      --bin=nd ^
      delete %1

@REM un poco de tiempo para que se acostumbren los nodos
timeout /t 10 /nobreak

@REM y finalmente actualizamos los nodos
docker compose up --detach --no-recreate --remove-orphans
