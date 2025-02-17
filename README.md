<img align="center" src="./media/img/logo/logo.png" width=560 height=130 />

<br/><br/><br/>

<hr style="width:35%" />

* [Integrantes](#integrantes)
* [Cómo Usar](#como-usar)
    - [Compilación](#compilación)
    - [Ejecución](#cómo-correr)
        * [Cliente](#cliente)
        * [Servidor](#servidor)
        * [Nodos](#nodos)
        * [Interfaz Gráfica](#interfaz-de-usuario)
        * [Simulador de Vuelos](#simulador-de-vuelos)
* [Cómo Testear](#cómo-testear)
* [Cómo Contribuir](#cómo-contribuir)

<hr style="width:35%" />

# Integrantes

| <center>Integrante</center> | <center>Padrón</center> | <center>Mail</center> | <center>GitHub</center> |
|:------------------------|:-----------------------:|:----------------------|:------------------------|
| **Franco Lighterman Reismann** | 106714| flighterman@fi.uba.ar | <img align="center" src="https://github.com/NLGS2907.png" height=32 width=32 /> [NLGS2907](https://github.com/NLGS2907) |
| **Matias Mendiola Daniel Escalante** | 110379 | mmendiolae@fi.uba.ar | <img align="center" src="https://github.com/MateXOre.png" height=32 width=32 /> [MateXOre](https://github.com/MateXOre) |
| **Francisco Antonio Pereyra** | 105666 | fpereyra@fi.uba.ar | <img align="center" src="https://github.com/fapereyra.png" height=32 width=32 /> [fapereyra](https://github.com/fapereyra) |
| **Victor Daniel Cipriano Sequera** | 106593 | vcipriano@fi.uba.ar | <img align="center" src="https://github.com/VicDC21.png" height=32 width=32 /> [VicDC21](https://github.com/VicDC21) |

# Como usar 

A continuación se detallan los pasos para compilar y ejecutar el programa.<br/>
El proyecto se divide en 8 _crates_ que se comunican entre sí.

En general, se usa la herramienta [`cargo`](https://doc.rust-lang.org/cargo/) para generar, correr y administrar el proyecto;
además de _lintear_ y formatear el mismo.

> **Nota:** _Se sobreentiende que los siguientes comandos se ejecutan en un entorno Linux/UNIX._

## Compilación

El proyecto se puede compilar completo con

```console
$ cargo build
```

o, si se desea compilar una _crate_ en concreto, se puede usar:

```console
$ cargo build -p <crate>
```

donde `<crate>` puede tomar uno de los siguientes valores:

* `client`
* `data`
* `interface`
* `parser`
* `protocol`
* `server`
* `simulator`
* `tokenizer`

## Cómo correr

El proyecto tiene soporte para correr los binarios de una _crate_ relevante.

No todas las _crates_ tienen un binario, y algunas tienen más de uno; en cuyo caso, todas las
_crates_ con binarios tienen una opción de elegir explícitamente cual binario correr, o en caso
de no especificar, cuentan con una configuración por defecto.

La sintaxis simple para correr el binario por defecto es:

```console
$ cargo run -p <crate>
```

y para correr un binario específico de esta:

```console
$ cargo run -p <crate> --bin <bin>
```

donde `<bin>` es un nombre del binario según la _crate_ que corresponda, y `<crate>` es, de nuevo,
un valor de los siguientes:

* `client`
* `data`
* `interface`
* `parser`
* `protocol`
* `server`
* `simulator`
* `tokenizer`

<br/>

<u><i><b>Detalles de los binarios por cada _crate_ se explican más abajo.</b></i></u>

<hr style="width:20%" />

### Cliente

#### `cli` ***(Default)***

Se puede invocar un [cliente simple](./client/src/bin/cli.rs) por consola.

```console
$ cargo run -p client --bin cli
```

**o, como es la opción por defecto,** esto también vale:

```console
$ cargo run -p client
```

### Servidor

#### `sv` ***(Default)***

Se da la opción de levantar todos los nodos de una, en una [sóla consola](./server/src/bin/sv.rs).

```console
$ cargo run -p server --bin sv [echo]
```

donde `echo` es opcional y se utiliza para inciar el servidor en modo ECHO.

**Adicionalmente, como es la opción por defecto,** esto también vale:

```console
$ cargo run -p server [echo]
```

#### `nd`

También hay soporte para levantar un [nodo aislado](./server/src/bin/nd.rs) por consola.
Para poder levantarse deben existir su ID e IP correspondientes en un archivo
llamado [`node_ips.csv`](./node_ips.csv) cuyas columnas son del estilo:

```csv
node_id,ip
...
```

<u><i>Además, el nodo con mayor ID debe ser levantado último.</i></u>

```console
$ cargo run -p server --bin nd [new] <id> <ip> [echo]
```

donde:

* `new` es una opción para agregar dinámicamente un nuevo nodo.
* `id` es el ID interno a usar para el nodo.
* `ip` es la IP a ser asignada al nodo.
* `echo` es otra opción para iniciar este nodo particular en modo ECHO.

### Interfaz de Usuario

Esta _crate_ es la que más dependencias utiliza, ya que se encarga de correr el
_FrontEnd_ de la aplicación.

#### `gui` ***(Default)***

El binario para correr la app en [modo gráfico](./interface/src/bin/gui.rs).

```console
$ cargo run -p interface --bin gui
```

**o, al ser la opción por defecto,** esto también vale:

```console
$ cargo run -p interface
```

### Simulador de Vuelos

El simulador cada tanto va insertando datos en los lugares relevantes para simular la creación
de vuelos en curso.

#### `sim` ***(Default)***

Un [menú interactivo](./simulator/src/bin/sim.rs) por consola para controlar la simulación.

```console
$ cargo run -p simulator --bin sim
```

**o, como es la opción por defecto,** lo siguiente también es válido:

```console
$ cargo run -p simulator
```

#### `demo`

Una [demostración](./simulator/src/bin/demo.rs) pre-hecha para correr en el simulador.
Culmina también en el menú.

```console
$ cargo run -p simulator --bin demo
```

# Cómo Testear

Los tests, siendo que se desee ejecutarlos manualmente, se puede con:

```console
$ cargo test --all-features
```

# Cómo contribuir

Las convenciones generales a seguir para contribuir en el proyecto quedan exhaustivamente
explicitadas en el [manifiesto](./CONTRIBUTING.md) relevante, y de otra forma dejadas a
libre interpretación. 
