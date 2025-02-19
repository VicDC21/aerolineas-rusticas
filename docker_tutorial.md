# Tutorial para Docker

[Docker](https://www.docker.com/) es una herramienta para correr aplicaciones en "contenedores",
y el proyecto soporta correr varios de sus componentes usando dichos contenedores.

A continuación se explica todo lo relevante a descargar imágenes, así como construir, correr,
detener y/o destruir contenedores para tanto los nodos como el cliente GUI.

> **Nota:** Las imágenes de Docker a utilizar vienen con su propia instancia de Rust. Esto permite que todos utilizen la misma versión, además que no hace falta que el usuario realize pasos extra si sólo se quiere usar el proyecto por Docker.

<hr style="width:35%" />

* [Prerequisitos](#prerequisitos)
* [Comandos Manuales](#forma-manual)
    - [Ver Imágenes/Contenedores (`docker ls`)](#ver-imágenes-yo-contenedores)
    - [Construir una imagen (`docker build`)](#construir-una-imagen)
    - [Correr el contenedor (`docker run`)](#correr-un-contenedor)
        * [_en pasos..._ (`docker create` + `docker start` + `docker attach`)](#correr-el-contenedor-en-pasos)
    - [Detener el contenedor (`docker stop`/`docker kill`)](#detener-el-contenedor)
    - [Remover el contenedor (`docker rm`/`docker prune`)](#remover-el-contenedor)
    - [Ejecutar comandos en el contenedor (`docker exec`)](#ejecutar-comandos-en-el-contenedor)
* [Utilizando `compose`](#usando-docker-compose)

<hr style="width:35%" />

# Prerequisitos

1. Descargar [Docker Desktop](https://docs.docker.com/desktop/). <br/> El proyecto usa varios componentes de Docker, como el motor [Docker Engine](https://docs.docker.com/engine/), el **Docker CLI client**, [Docker Build](https://docs.docker.com/build/) y [Docker Compose](https://docs.docker.com/compose/) en varias partes de este tutorial. <br/> Sin embargo, como no se garantiza que la lista de componentes es exhaustiva, se prefiere usar **Docker Desktop** porque incluye todos estos componentes y más en una sóla instalación. <br/><br/> Particularmente, se recomienda **Docker Desktop** para [Linux](https://docs.docker.com/desktop/setup/install/linux/) o bien [Windows con WSL](https://docs.docker.com/desktop/setup/install/windows-install/) que son las versiones con las que se desarrolló el proyecto.

2. _(Opcional)_ Descargar la [imagen](https://hub.docker.com/_/rust) oficial de Docker para el lenguaje Rust. <br/> Particularmente la versión `1.84-slim`:
    ```console
    $ docker pull rust:1.84-slim
    ```
    _Nótese que este paso es opcional, ya que igual se descarga automáticamente la primera vez que se hace_ `build`.

# Forma Manual

Si no se desea utilizar [`docker compose`](#usando-docker-compose) o si se desea realizar una depuración, a continuación se explican los comandos manuales.

En general, siempre se [_buildea_](#construir-un-contenedor) la imagen, se [corre](#correr-un-contenedor) un contenedor basado en esa imagen, y luego se lo [detiene](#detener-el-contenedor) o [elimina](#remover-el-contenedor) según convenga.

## Ver imágenes y/o contenedores

En cualquier momento se puede utilizar

```console
$ docker image ls
```
<details>
    <summary>
        <b>Salida de ejemplo</b>
    </summary>

```
REPOSITORY   TAG         IMAGE ID       CREATED       SIZE
rust         1.84-slim   988feec34834   2 weeks ago   810MB
rust         1.84        cdb394ef6c0d   2 weeks ago   1.47GB
```
</details>
<br/>
para ver las imágenes creadas, o

```console
$ docker container ls
```
<details>
    <summary>
        <b>Salida de ejemplo</b>
    </summary>

```
CONTAINER ID   IMAGE     COMMAND                  CREATED          STATUS          PORTS     NAMES
28a50508a3da   nodos     "cargo run --package…"   7 seconds ago    Up 6 seconds              nodo-13
0f8cc0b1d01c   nodos     "cargo run --package…"   13 seconds ago   Up 12 seconds             nodo-12
acf59190c191   nodos     "cargo run --package…"   21 seconds ago   Up 20 seconds             nodo-11
2a58eb8970e3   nodos     "cargo run --package…"   30 seconds ago   Up 30 seconds             nodo-10
```
</details>
<br/>
para ver los contenedores actualmente corriendo, su estado, etc.

## Construir una imagen

Primero necesitamos generar la **imagen**, que será la base de todos los contenedores que utilicen la misma lógica. Por ejemplo, se ha de crear una "plantilla" sobre la que construir cada contenedor de nodo.

### `nd`

Para construir una imagen de nodo, basta con hacer:

```console
$ docker build -t <nombre> -f docker/nd/Dockerfile .
```

donde `<nombre>` es el nombre de la "plantilla" o **imagen** que vamos crear.


### `gui`

Similarmente, existe otro manifiesto para crear el binario de la interfaz por separado:

```console
$ docker build -t <nombre> -f docker/gui/Dockerfile .
```

## Correr un contenedor

Aquí es cuando creamos los contenedores en base a la plantilla que ya hicimos.

```console
$ docker run -it [-d] --rm --name <contenedor> <imagen> [args...]
```

donde:

* `<imagen>` es el nombre de una **imagen** _ya creada_ a ser usada como "plantilla".
* `<contenedor>` es el nombre del contenedor a crear en base a `<imagen>`.
* `-d` es un argumento que, de estar presente, delega la ejecución del contenedor a un proceso de fondo en vez de abrirlo en nuestra misma consola.
* `[args...]` son los argumentos que de otra manera se llamarían como si fuese un comando normal para el binario. <br/> Por ejemplo, para el binario `nd` que normalmente sería:
    ```console
    $ cargo run -p server --bin nd [new] <id> <ip> [echo]
    ```
    ahora es
    ```console
    $ docker run -it --rm --name <contenedor> <imagen> [new] <id> <ip> [echo]
    ```
    suponiendo que `<imagen>` en este caso es una de tipo [`nd`](#nd).

### Correr el contenedor en pasos

`docker run` _es, en esencia, equivalente a_ [`docker create`](#para-crear-el-contenedor) + [`docker start`](#para-iniciar-el-contenedor) + [`docker attach`](#para-abrir-el-contenedor), y sirve como método de conveniencia para evitar invocar los tres por separado.

> **Nota:** El último paso de `docker attach` se puede evitar con el argumento `-d`.

Por supuesto, siempre se pueden ejecutar las partes como comandos separados.


### Para crear el contenedor

```console
$ docker container create --name <contenedor> <imagen>
```

### Para iniciar el contenedor

```console
$ docker container start [-a] <contenedor>
```

con `-a` disponible si también se quiere [abrir](#para-abrir-el-contenedor) el contenedor.

### Para abrir el contenedor

```console
$ docker container attach <contenedor>
```

> **Nota:** Esto sólo permite ver el STDOUT del contenedor. Esto NO abre una consola interactiva, cosa que se logra mejor con [`docker exec`](#ejecutar-comandos-en-el-contenedor).

## Detener el contenedor

Nótese que en ejemplo de más arriba se utiliza el argumento `--rm`, que elimina el contenedor apenas éste "sale" de ejecución por cualquier motivo.

Pero en caso de incluir esta opción, o bien si se cuelga el contenedor, está la opción de detener el mismo de una forma con gracia:

```console
$ docker container stop <contenedor>
```

o para forzar el cierre:

```console
$ docker container kill <contenedor>
```

En ambos casos, **esto sólo provoca que un contenedor pase a estar de un estado "corriendo" a uno "sin uso",** pero todavía existe en el sistema.

## Remover el contenedor

Para eliminar contenedores, basta con:

```console
$ docker container rm [-f] <contenedor>
```

con `-f` como argumento opcional para forzar el proceso.

Alernativamente, existe:

```console
$ docker container prune [-f] [--filter [...]]
```

que elimina automáticamente todos los contenedores que no están en uso, y otorga más control sobre qué contenedores filtrar.

## Ejecutar comandos en el contenedor

Permite ejecutar comando en un contenedor **ya corriendo:**

```console
$ docker container exec -it [-d] <contenedor> <comando> [args...]
```

con `-d` para correrlo en el fondo en vez de abrirlo inmediatamente.

# Usando `docker compose`