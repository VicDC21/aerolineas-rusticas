<img align="center" src="./media/img/logo/logo.png" width=560 height=130 />

<br/><br/><br/>

<hr style="width:35%" />

* [Integrantes](#integrantes)
* [Cómo Usar](#como-usar)
    - [Compilación](#compilación)
    - [Ejecución](#cómo-correr)
        * [Cliente](#cliente)
        * [Servidor](#servidor)
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

A continuación se detallan los pasos para compilar y ejecutar el programa.

En general, se usa la herramienta [`cargo`](https://doc.rust-lang.org/cargo/) para generar, correr y administrar el proyecto;
además de _lintear_ y formatear el mismo.

> **Nota:** _Se sobreentiende que los siguientes comandos se ejecutan en un entorno Linux/UNIX._

## Compilación

El proyecto se compila dependiendo de la _feature_ que [sea necesaria](#cómo-correr), pero una
compilación general alcanza con:

```console
$ cargo build --all-features
```

## Cómo correr

El proyecto tiene soporte para correr sus diferentes partes.
Si se desea correr una sóla parte a la vez, basta con correr la sintaxis simple:

```console
$ cargo run [cli | --features "gui" gui | sim | sv [echo]]
```

o, si se desea correr por ejemplo `sv` _y luego_ `gui`, el proyecto puede ejecutar sus partes
en binarios separados:

```console
$ cargo run --bin [cli | gui | sim | sv]
```

ya que sino, se tratará de construir dos veces el mismo ejecutable, lo cual no siempre es posible.

### Cliente

Se puede invocar un cliente simple por consola.

<details>
    <summary>
        <b>Forma simple</b>
    </summary>

```console
$ cargo run cli
```

</details>

<details>
    <summary>
        <b>Binario Separado</b>
    </summary>

```console
$ cargo run --bin cli
```

</details>

### Servidor

También conocido como los "nodos".

<details>
    <summary>
        <b>Forma simple</b>
    </summary>

```console
$ cargo run sv
```

</details>

<details>
    <summary>
        <b>Binario Separado</b>
    </summary>

```console
$ cargo run --bin sv
```

</details>

### Nodos

Se debe decidir entre correr el _Servidor_, que levanta todos los nodos en una sola consola, o los nodos indivualmente, los cuales se deben levantar de a uno por consola.
Para poder levantarse deben existir su ID e IP correspondientes en un archivo llamado `node_ips.csv` cuyas columnas son `node_id,ip`. Además, el nodo con mayor ID debe ser levantado último.

<details>
    <summary>
        <b>Forma simple</b>
    </summary>

```console
$ cargo run nd <id>
```

</details>

<details>
    <summary>
        <b>Binario Separado</b>
    </summary>

```console
$ cargo run --bin nd <id>
```

</details>

### Interfaz de Usuario

La interfaz de usuario incluye muchas dependencias y por lo tanto, se compila
con una _feature_ aparte con la forma `--features "gui"` o `--features gui`.

<details>
    <summary>
        <b>Forma simple</b>
    </summary>

```console
$ cargo run --features "gui" gui
```

</details>

<details>
    <summary>
        <b>Binario Separado</b>
    </summary>

```console
$ cargo run --features "gui" --bin gui
```

</details>

> **Nota:** Como `gui` es la única _feature_ que tiene el proyecto, `--features "gui"` se puede
> reemplazar por `--all-features`

### Simulador de Vuelos

El simulador cada tanto va insertando datos en los lugares relevantes para simular la creación
de vuelos en curso.

<details>
    <summary>
        <b>Forma simple</b>
    </summary>

```console
$ cargo run sim
```

</details>

<details>
    <summary>
        <b>Binario Separado</b>
    </summary>

```console
$ cargo run --bin sim
```

</details>

# Cómo Testear

Los tests, siendo que se desee ejecutarlos manualmente, se puede con:

```console
$ cargo test --all-features
```

# Cómo contribuir

Las convenciones generales a seguir para contribuir en el proyecto quedan exhaustivamente
explicitadas en el [manifiesto](./CONTRIBUTING.md) relevante, y de otra forma dejadas a
libre interpretación. 
