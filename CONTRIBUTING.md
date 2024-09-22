# **Cómo contribuir**

El proyecto es desarrollado en el lenguaje [Rust](https://www.rust-lang.org/) en su versión más
reciente _(1.81.0 al momento de escribir esto)_, junto con todas las herramientas
(`rustc`, `rustup`, `cargo`, _etc._) que una instalación estándar suele traer. <br/>
Mientras la gran mayoría de cosas relacionadas al código fuente son relegadas a dichas herramientas,
algunas convenciones para la coordinación en el repositorio son acá explicitadas.

<u>Dichas convenciones deberían seguirse y pueden ser motivo de
rechazar o invalidad un *Pull Request*/*issue*.</u>

> **Nota:** _Si bien las funciones de estas herramientas pueden ser agnósticas al sistema operativo, siempre se preferirá trabajar sobre un sistema Linux/UNIX._

<hr width="30%" align="left" />

# Índice

* [Código Fuente](#código-fuente)
    - [Formato](#formato)
    - [*Linting*](#linting)
    - [Módulos](#módulos)
    - [Documentación](#documentación)
    - [*Tests*](#tests)
        * [*Doctests*](#doctests)
        * [*Tests* Unitarios](#unit-tests)

* [Repositorio de GitHub](#repositorio)
    - [*Pull Requests*](#pull-requests)
        * [Motivación](#motivación)
        * [Cambios Hechos](#cambios-hechos)
        * [Lista de cambios](#lista-de-cambios)
        * [*Checklist*](#checklist)

    - [*Commits*](#commits)

    - [Ramas / *Branches*](#branches)

    - [*Issues*](#issues)

<hr width="30%" align="left" />

# Código Fuente

En términos del código fuente, se relegan varias tareas a las herramientas pertinentes.

## Formato

Se contempla un formato tal cual y como lo deja la herramienta `cargo fmt`. Se ejecuta desde la
raíz del proyecto:
```console
$ cargo fmt
```
para formatear todos los archivos. <br/>
En caso de querer afectar archivos específicos, usar `rustfmt` en su lugar.

## *Linting*

Similarmente, se debe usar `cargo check` o *clippy* para verificar el código. Se prefiere la
segunda opción:
```console
$ cargo clippy --all-targets --all-features
```

## Módulos

Cada "unidad de compilación", como _structs_ o _enums_ deben ir en sus propios módulos separados.

<details>
<summary><b>Ejemplos</b></summary>
Por ejemplo, los elementos:

```rs
pub struct FooStruct {
    pub a: String,
    pub b: i32
}

pub struct BarStruct {
    pub c: String,
    pub d: i32
}

pub enum FoobarEnum {
    Foo(FooStruct),
    Bar(BarStruct)
}
```
podrían ser atomizados de la siguiente manera:

```
├─ src/
│   ├─main.rs
│   ├─lib.rs
│   │
│   ├─foobar/
│   │   ├─mod.rs
│   │   ├─foo.rs
│   │   └─bar.rs
│   │
│   └─enums/
│       ├─mod.rs
│       └─foobar_enum.rs
│
└─[...]
```
con:
```rs
//! src/lib.rs
pub mod foobar;
pub mod enums;
```
```rs
//! src/foobar/mod.rs
pub mod foo;
pub mod bar;
```
```rs
//! src/enums/mod.rs
pub mod foobar_enum;
```
```rs
//! src/enums/foobar_enum.rs
use crate::foobar::{foo::FooStruct, bar::BarStruct};

pub enum FoobarEnum {
    Foo(FooStruct),
    Bar(BarStruct)
}
```

</details>

## Documentación

Se deberá documentar cada paquete, módulo, función, _struct_, etc. Siguiendo las
[convenciones](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) de
[`cargo doc`](https://doc.rust-lang.org/rust-by-example/meta/doc.html).

Se deberá también incluir el atributo `#![warn(missing_docs)]` al principio del archivo
[`lib.rs`](./src/lib.rs) para recordar a los integrantes si falta dicha documentación en ciertos
elementos.

## *Tests*

Los tests se pueden ejecutar todos con:
```console
$ cargo test --verbose
```

### *Doctests*

En caso de que la [documentación](#documentación) incluya _snippets_ de código de demostración,
los *tests* podrán tratar de compilarlos para comprobar su integridad.

<details>
<summary><b>Ejemplos</b></summary>

```rs
//! src/prueba.rs de proyecto "foobar"

/// Prueba a tirar un número.
///
/// ```rust
/// # use foobar::prueba::probar;
/// assert_eq!(-1, probar(true));
/// assert_eq!(1, probar(false));
/// ```
pub fn probar(neg: bool) -> i32 {
    if neg {-1} else {1}
}
```

</details>

### *Unit Tests*

Si se conviene que es relevante comprobar el funcionamento de ciertos elementos, los _tests_
unitarios deben ir en el mismo [módulo](#módulos) que dicho elemento, e incluirlos con un módulo
llamado "tests", con la configuración `#[cfg(test)]` _(para que sólo sea compilada en
`cargo test` y no otro comando)_, <br/>
y con la nomenclatura de funciones `test_<num>_<desc>()` sin argumentos y con la configuración `#[test]`.

<details>
<summary><b>Ejemplos</b></summary>

```rs
//! src/enums/foobar_enum.rs
use crate::foobar::{foo::FooStruct, bar::BarStruct};

pub enum FoobarEnum {
    Foo(FooStruct),
    Bar(BarStruct)
}

#[cfg(test)]
mod tests {
    // Es a este punto que se deben incluir todos los imports relevantes a los tests.
    // Cuidado que deben ser imports desde la raíz de la librería, y usando el prefijo `crate`.
    use crate::enums::foobar_enum::FoobarEnum;
    use crate::foobar::{foo::FooStruct, bar::BarStruct};

    #[test]
    fn test_1_es_foo() {
        let foo = FooStruct {a: "a", b: 20};
        assert!(matches!(FoobarEnum::Foo, FoobarEnum(foo)));
    }

    #[test]
    fn test_2_es_bar() {
        let bar = BarStruct {c: "c", d: 35};
        assert!(matches!(FoobarEnum::Bar, FoobarEnum(bar)));
    }
}
```

</details>

<hr/>

# Repositorio

He aquí las convenciones pertinentes a la colaboración del proyecto tal que los cambios sean
organizados en elementos disponibles en el [repositorio](https://github.com/taller-1-fiuba-rust/24C2-Los-Aldeanos-Panas) de GitHub.

> Posiblemente, desde donde se esté leyendo esto.

## *Pull Requests*

Los *pull requests* o "PRs" deben seguir el estilo de la
[plantilla](./templates/pull_requests/pr_template.md) diseñada para tal fin. <br/>
En ella, se encuentran las siguientes secciones:

### Motivación

Acá debería ir una descripción breve del PR y el propósito que pretende cumplir.

### Cambios Hechos

Una descripción breve de los cambios hechos, y cómo podrían afectar desde ahora al código con el
que interactúa.

### Lista de cambios

Una lista (sino es exhaustiva, que comprenda los puntos más importantes), detallando los cambios
precisos hechos en el código u en otro lugar.

* Normalmente
* tendrá un
* formato
    - más o
    - menos
        * parecido
* a esto.

### *Checklist*

He aquí una casilla de casos más comunes para llenar en todo modelo de PRs. En caso de querer
agregar más para el PR en cuestión se puede, pero no quitar alguna de las que ya vienen en la
plantilla. <br/>

> **Nota:** *El estilo que aparece en GitHub parece exclusivo de esta página y no funciona sólo con MarkDown,
así que a continuación se detalla la forma de crear y editar algunas.*

```md
## Casillas vacías:

*(válidas)*
* [ ]
- [ ]

*(inválidas)*
* []
- []
* [asd]

## Casillas "llenas":

* [x]
* [X]
- [x]
- [X]
```

**Todos** los *pull requests* deberían ir acompañados de un "asignado" _(asignee)_, etiquetas, y
otros datos correspondientes cuando se lo publica en GitHub. Opcionalmente, también podría estar
asociado a un *issue*.

<hr/>

## *Commits*

Los títulos de los *commits* deben de ser precedidos por `"<categoria>: "` de la manera:
```
<categoria>: Título del commit
```
Donde `<categoria>` referie a uno de los siguientes casos:

* **feat:** Una nueva *feature*.
* **fix:** Un arreglo de un *bug*.
* **docs:** Cambios en la documentación.
* **style:** Cambios que no afectan al código de manera funcional.
* **refactor:** Cambios que no arreglan errores o agregan *features*.
* **test:** Cambios que agregan tests.
* **chore:** Cambios hechos a programas auxiliares del proyecto, como la compilación automática
del programa.

Si no se identifica la ocasión con uno de estos casos, se puede evitar el prefijo. <br/>
**No es necesario,** pero se prefiere usar palabras en minúsculas, salvo que se refiera objetos que
específicamente requieren otra forma.

<details>
<summary><b>Ejemplos</b></summary>

```
feat: agregado un server
```
```
fix: arreglado el bug del server
```
```
docs: agregada documentación del server
```
```
test: agregados tests unitarios para el cliente (bueno, y también para el server)
```

</details>
<br/>

**No es obligatorio** que los *commits* tengan una descripción, pero sí un título breve.

<hr/>

## *Branches*

Las ramas del repositorio, o *"branches"*, deberían ser creadas siguiendo una convención:
```
<categoria>/<titulo>
```
donde `<categoria>` puede ser cualquiera de los mismos tipos explicados [arriba](#commits), y el
título puede ser cualquier cosa, incluso otras subdivisiones. <br/>
Así, las ramas mismas están divididas por categoría.

> **Nota:** Es posible que algunos _workflows_ de GitHub no funcionen si la rama no es nombrada apropiadamente.

<details>
<summary><b>Ejemplos</b></summary>

```console
$ git branch feat/server
```
```console
$ git branch docs/docstrings
```
```console
$ git branch docs/readme/simple-makeover
```
```console
$ git branch docs/readme/contributing
```

</details>

<hr/>

## *Issues*

Las *issues* no son obligatorias de usar, pero dada la ocasión, deberán seguir una plantilla según
el [caso](./templates/issues/) que convenga.
De no estar contemplado el caso en una plantilla, se puede seguir un
estilo libre (pero se espera uno similar). <br/>

Los casos en cuestión son:

* 🐛 [Reportar un error](./templates/issues/bug_report_template.md)
* 🎨 [Una idea de diseño](./templates/issues/design_idea_template.md)
* 📚 [Una mejora de la documentación](./templates/issues/docs_augmentation_template.md)
* 🚀 [Una idea de *feature*](./templates/issues/feature_request_template.md)
* 🚧 [Una ocasión en la que refactorizar código](./templates/issues/refactor_code_template.md)

Donde el título del *issue* **debe empezar sí o sí** con el emoji correspondiente a esa categoría.
De no entrar en ninguna, el *issue* de estilo "libre" puede incluir cualquier emoji que no sea
uno de esos. <br/>

*En lo posible,* tratar de encajar la necesidad en alguna de esa categorías. **Por ejemplo:** un
reporte de una vulnerabilidad de seguridad podría ir acompañada de una refactorización, entonces
caería en la categoría 🚧; también, agregar librerías o extensiones para compilar el proyecto u
otras operaciones externas bien podrían ser 📚 o 🚀.
