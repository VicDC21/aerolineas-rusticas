# Tutorial para Docker

[Docker](https://www.docker.com/) es una herramienta para correr aplicaciones en "contenedores",
y el proyecto soporta correr varios de sus componentes usando dichos contenedores.

A continuación se explica todo lo relevante a descargar imágenes, así como construir, correr,
detener y/o destruir contenedores para los nodos.

> [!NOTE]
> Las imágenes de Docker a utilizar vienen con su propia instancia de Rust. Esto permite que todos los nodos utilizen la misma versión, además que no hace falta que el usuario realize pasos extra si sólo se quiere hostear los nodos por Docker.

<hr style="width:35%" />

* [Prerequisitos](#prerequisitos)
* [Comandos Manuales](#forma-manual)
    - [Ver Imágenes/Contenedores (`docker ls`)](#ver-imágenes-yo-contenedores)
    - [Construir una imagen (`docker build`)](#construir-una-imagen)
    - [Detener un contenedor (`docker stop`/`docker kill`)](#detener-un-contenedor)
    - [Remover el contenedor (`docker rm`/`docker prune`)](#remover-el-contenedor)
    - [Ejecutar comandos en el contenedor (`docker exec`)](#ejecutar-comandos-en-el-contenedor)
* [Utilizando `compose`](#usando-docker-compose)
    - [Levantando el clúster de nodos](#levantando-el-clúster)
    - [Deteniendo los nodos](#deteniendo-el-clúster)
    - [Apagando los nodos](#cerrando-el-clúster)
    - [Modificar nodos de forma dinámica](#modificando-nodos-dinámicamente)
    - [Viendo el _output_](#viendo-el-output-de-los-nodos)

<hr style="width:35%" />

# Prerequisitos

1. Descargar [Docker Desktop](https://docs.docker.com/desktop/). <br/> El proyecto usa varios componentes de Docker, como el motor [Docker Engine](https://docs.docker.com/engine/), el **Docker CLI client**, [Docker Build](https://docs.docker.com/build/) y [Docker Compose](https://docs.docker.com/compose/) en varias partes de este tutorial. <br/> Sin embargo, como no se garantiza que la lista de componentes es exhaustiva, se prefiere usar **Docker Desktop** porque incluye todos estos componentes y más en una sóla instalación. <br/><br/> Particularmente, se recomienda **Docker Desktop** para [Linux](https://docs.docker.com/desktop/setup/install/linux/) o bien [Windows con WSL](https://docs.docker.com/desktop/setup/install/windows-install/) que son las versiones con las que se desarrolló el proyecto.

2. _(Opcional)_ Descargar la [imagen](https://hub.docker.com/_/rust) oficial de Docker para el lenguaje Rust. <br/> Particularmente la versión `1.84-slim`:
    ```console
    $ docker pull rust:1.84-slim
    ```
    > [!NOTE]
    > _Nótese que este paso es opcional, ya que igual se descarga automáticamente la primera vez que se hace_ `build`.

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


### `full`

Similarmente, existe otro manifiesto para copiar el directorio tal cual y realizar pruebas dentro:

```console
$ docker build -t <nombre> -f docker/full/Dockerfile .
```

## Detener un contenedor

Normalmente se utiliza el argumento `--rm`, que elimina el contenedor apenas éste "sale" de ejecución por cualquier motivo.

Pero en caso de incluir esta opción, o bien si se cuelga el contenedor, está la opción de detener el mismo de una forma con gracia:

```console
$ docker container stop <contenedor>
```

o para forzar el cierre:

```console
$ docker container kill <contenedor>
```

En ambos casos, **esto sólo provoca que un contenedor pase a estar de un estado "corriendo" a uno "sin uso",** pero todavía existe como instancia "dormida" o "apagada" en la colección de Docker.

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

Usar [Docker Compose](https://docs.docker.com/compose/) resulta mucho más sencillo a nuestros propósitos.

El [mainifiesto](./compose.yaml) relevante se encarga de crear la imagen de los nodos relevante si no existe todavía, y luego levantar el clúster de nodos

## Levantando el clúster

Para construir y levantar el clúster de nodos, basta con hacer:

```console
$ docker compose up
```

> [!TIP]
> Este genera un _output_ que va mostrando los logs de cada nodo, pero el mismo no se actualiza con modificaciones al clúster.
>
> Si se planea agregar o sacar nodos posteriormente, se recomienda utilizar `docker compose -d` o `docker compose --detach` en su lugar para correrlo en segundo plano.
> 
> Posteriormente, se va a especificar una forma de ver los logs de igual manera.

## Deteniendo el clúster

Si se quiere detener (pero no borrar) el clúster de nodos, el comando es:

```console
$ docker compose stop
```

## Cerrando el clúster

En cualquier momento, desde una consola se puede hacer:

```console
$ docker compose down
```

para apagar todos los contenedores, **y luego eliminarlos,** todo de forma bonita.

## Modificando Nodos dinámicamente

Siempre se puede modificar la cantidad de nodos del clúster de forma dinámica,
más allá de la cantidad inicial.
La cantidad de nodos al momento de apagarse persiste entre sesiones.

### Agregando

Se debe ir a otra consola (o la misma si se levantó el clúster con `--detach`), y ejecutar el _script_ relevante dependiendo del sistema operativo en el que uno se encuentre.

> [!NOTE]
> Recordar que estos comandos se corren desde la raíz del proyecto.

#### Windows

```bat
./docker/add.bat <id> <ip>
```

#### UNIX/Linux

```sh
./docker/add.sh <id> <ip>
```

donde:

* `<id>` es un número entre 15 y 255 y será el ID del nuevo nodo, y el número por el que se lo identificará con comandos posteriores.
* `<ip>` Es una IP en formato IPv4 con rango desde `192.0.0.0` hasta `255.255.255.255`. **No se garantizan que funcionen TODAS las combinaciones** porque puede que alguna esté reservada por el sistema, pero funciona igualmente con un gran número de combinaciones arbitrarias.

### Borrando

De manera análoga, los _scripts_ para sacar un nodo son:

#### Windows

```bat
./docker/remove.bat <id>
```

#### UNIX/Linux

```sh
./docker/remove.sh <id>
```

con `<id>` siendo el ID del nodo interesado.


Estos _scripts_ se encargan de toda la "magia" para preparar el _compose_ y todos los archivos relevantes para modificar el clúster con otra cantidad de nodos.

### Viendo el _output_ de los nodos

Como se [explicó anteriormente](#levantando-el-clúster), `docker compose --detach` es recomendado porque levanta el clúster en segundo plano. De ser éste el caso, se puede hacer luego:

```console
$ docker compose logs -f
```

Para levantar la misma ventana de _logging_ de _output_ de nodos que de otra forma se mostraría igualmente sin el `--detach`.

La diferencia radica en que, si cambiamos la cantidad de nodos, podemos cerrar esta vista de _logging_ de forma segura sin arriesgar a apagar el clúster, y volver a abrirlo para reflejar los cambios y mostrar el _output_ de los nuevos nodos también.

`docker compose up` corre el clúster en ese mismo proceso, y uno no puede salir de esa vista sin cerrar el clúster primero, ni siquiera para reflejar cambios en la cantidad de nodos.
