# Taller de Programacion - "Los Aldeanos Panas"

## Integrantes

| <center>Integrante</center> | <center>Padrón</center> | <center>Mail</center> | <center>GitHub</center> |
|:------------------------|:-----------------------:|:----------------------|:------------------------|
| **Franco Lighterman Reismann** | 106714| flighterman@fi.uba.ar | <img align="center" src="https://github.com/NLGS2907.png" height=32 width=32 /> [NLGS2907](https://github.com/NLGS2907) |
| **Matias Mendiola Daniel Escalante** | 110379 | mmendiolae@fi.uba.ar | <img align="center" src="https://github.com/MateXOre.png" height=32 width=32 /> [MateXOre](https://github.com/MateXOre) |
| **Francisco Antonio Pereyra** | 105666 | fpereyra@fi.uba.ar | <img align="center" src="https://github.com/fapereyra.png" height=32 width=32 /> [fapereyra](https://github.com/fapereyra) |
| **Victor Daniel Cipriano Sequera** | 106593 | vcipriano@fi.uba.ar | <img align="center" src="https://github.com/VicDC21.png" height=32 width=32 /> [VicDC21](https://github.com/VicDC21) |

## Como usar 

A continuación se detallan los pasos para compilar y ejecutar el programa.

En general, se usa la herramienta [`cargo`](https://doc.rust-lang.org/cargo/) para generar, correr y administrar el proyecto;
además de _lintear_ y formatear el mismo.

> **Nota:** _Se sobreentiende que los siguientes comandos se ejecutan en un entorno Linux/UNIX._

### Compilación

El proyecto se compila haciendo:

```console
$ cargo build
```

### Cómo correr

El proyecto tiene soporte para correr sus diferentes partes.

#### Servidor

También conocido como los "nodos", se invocan con:

```console
$ cargo run sv
```

#### Cliente

Una instancia de cliente en consola se puede invocar con:

```console
$ cargo run cli
```

#### Interfaz de Usuario

La interfaz de usuario incluye muchas dependencias y por lo tanto, se compila
con una _feature_ aparte. El comando es:

```console
$ cargo run --features "gui" gui
```

## Cómo testear

Los tests, siendo que se desee ejecutarlos manualmente, se puede con:

```console
$ cargo test
```

## Cómo contribuir

Las convenciones generales a seguir para contribuir en el proyecto quedan exhaustivamente
explicitadas en el [manifiesto](./CONTRIBUTING.md) relevante, y de otra forma dejadas a
libre interpretación. 
