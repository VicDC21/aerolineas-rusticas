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

    (
        FOR /F "tokens=* delims=" %%l in (%~1) DO (
            IF "%%l" NEQ "%~2" (
                echo %%l
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

@REM borramos la IP del CSV del cliente
CALL:borrar_linea "./client_ips.csv" "%1,127.0.0.%1"

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
docker compose up --detach --no-recreate
