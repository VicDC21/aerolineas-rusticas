//! Módulo para mensajes de errores.

use std::{collections::HashMap, fmt::{Display, Formatter, Result}, net::IpAddr};

use crate::cassandra::{notations::consistency::Consistency, traits::Byteable};

/// La forma del mensaje de error es `<code><message>[...]`.
/// Luego, dependiendo del código de error, tendrá más información o no luego del mensaje.
pub enum Error {
    /// Un error del lado del servidor.
    ServerError(String),

    /// Un mensaje del cliente ocasionó una violación de protocolo.
    ProtocolError(String),

    /// La autenticación era requerida y falló.
    AuthenticationError(String),

    /// Un nodo no se encontraba disponible para responder a la query.
    ///
    /// El resto del mensaje es `<cl><required><alive>`, donde:
    /// * `<cl>` es el nivel de [Consistency](crate::cassandra::notations::consistency::Consistency) de la query que lanzó esta excepción.
    /// * `<required>` es un número ([i32]) que representa la cantidad de nodos que deberían estar disponibles para respetar `<cl>`.
    /// * `<alive>` es un número ([i32]) que representa la cantidad de réplicas que se sabía que estaban disponibles cuando el request había sido procesado (como se lanzó ésta excepción, se sabe que `<alive> < <required>`).
    UnavailableException(String, Consistency, i32, i32),

    /// El request no puede ser procesado porque el nodo coordinador está sobrecargado.
    Overloaded(String),

    /// El request fue de lectura pero el nodo coordinador estaba en proceso de boostrapping (inicialización).
    IsBootstrapping(String),

    /// Un error de trucamiento.
    TruncateError(String),

    /// Timeout exception durante un request de escritura.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><writeType><contentions>`, donde:
    /// * `<cl>` es el nivel de [Consistency](crate::cassandra::notations::consistency::Consistency) de la query que lanzó esta excepción.
    /// * `<received>` es un número ([i32]) que representa la cantidad de nodos que han reconocido la request.
    /// * `<blockfor>` es un número ([i32]) que representa la cantidad de réplicas cuya confirmación es necesaria para cumplir `<cl>`.
    /// * `<writeType>` es un [String] que representa el tipo de escritura que se estaba intentando realizar. El valor puede ser:
    ///     * "SIMPLE": La escritura no fue de tipo batch ni de tipo counter.
    ///     * "BATCH": La escritura fue de tipo batch (logged). Esto signifca que el log del batch fue escrito correctamente, caso contrario, se debería haber enviado el tipo "BATCH_LOG".
    ///     * "UNLOGGED_BATCH": La escritura fue de tipo batch (unlogged). No hubo intento de escritura en el log del batch.
    ///     * "COUNTER": La escritura fue de tipo counter (batch o no).
    ///     * "BATCH_LOG": El timeout ocurrió durante la escritura en el log del batch cuando una escritura de batch (logged) fue pedida.
    ///     * "CAS": El timeout ocurrió durante el Compare And Set write/update (escritura/actualización).
    ///     * "VIEW": El timeout ocurrió durante una escritura que involucra una actualización de VIEW (vista) y falló en adquirir el lock de vista local (MV) para la clave dentro del timeout.
    ///     * "CDC": El timeout ocurrió cuando la cantidad total de espacio en disco (en MB) que se puede utilizar para almacenar los logs de CDC (Change Data Capture) fue excedida cuando se intentaba escribir en dicho logs.
    /// * `<contentions>` es un número ([u16]) que representa la cantidad de contenciones ocurridas durante la operación CAS. Este campo solo se presenta cuando el <writeType> es "CAS".
    ///
    /// TODO: _Quizás meter writeType en un enum._
    WriteTimeout(String, Consistency, i32, i32, String, Option<u16>),

    /// Timeout exception durante un request de lectura.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><data_present>`, donde:
    /// * `<cl>` es el nivel de [Consistency](crate::cassandra::notations::consistency::Consistency) de la query que lanzó esta excepción.
    /// * `<received>` es un número ([i32]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([i32]) que representa la cantidad de réplicas cuya respuesta es necesaria para cumplir `<cl>`. Notar que es posible tener `<received> >= <blockfor>` si <data_present> es false. También en el caso (improbable) donde <cl> se cumple pero el nodo coordinador sufre un timeout mientras esperaba por la confirmación de un read-repair.
    /// * `<data_present>` es un [u8] (representa un booleano: 0 es false, distinto de 0 es true) que indica si el nodo al que se le hizo el pedido de la data respondió o no.
    ReadTimeout(String, Consistency, i32, i32, u8),

    /// Una excepción de lectura que no fue ocasionada por un timeout.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><reasonmap><data_present>`, donde:
    /// * `<cl>` es el nivel de [Consistency](crate::cassandra::notations::consistency::Consistency) de la query que lanzó esta excepción.
    /// * `<received>` es un número ([i32]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([i32]) que representa la cantidad de réplicas cuya respuesta es necesaria para cumplir `<cl>`.
    /// * `<reasonmap>` es un "mapa" de endpoints a códigos de razón de error. Esto mapea los endpoints de los nodos réplica que fallaron al ejecutar la request a un código representando la razón del error. La forma del mapa es empezando con un [i32] n seguido por n pares de <endpoint><failurecode> donde <endpoint> es un [IpAddr](std::net::IpAddr) y <failurecode> es un [u16].
    /// * `<data_present>` es un [u8] (representa un booleano: 0 es false, distinto de 0 es true) que indica si el nodo al que se le hizo el pedido de la data respondió o no.
    ReadFailure(String, Consistency, i32, i32, HashMap<IpAddr, u16>, u8),

    /// Una función (definida por el usuario) falló durante su ejecución.
    ///
    /// El resto del mensaje es `<keyspace><function><arg_types>`, donde:
    /// * `<keyspace>` es un [String] representando el _keyspace_ en el que se encuentra la función.
    /// * `<function>` es un [String] representando el nombre de la función.
    /// * `<arg_types>` es una lista de [String] representando los tipos (en tipo CQL) de los argumentos de la función.
    FunctionFailure(String, String, String, Vec<String>),

    /// Una excepción de escritura que no fue ocasionada por un timeout.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><reasonmap><write_type>`, donde:
    /// * `<cl>` es el nivel de [Consistency](crate::cassandra::notations::consistency::Consistency) de la query que lanzó esta excepción.
    /// * `<received>` es un número ([i32]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([i32]) que representa la cantidad de réplicas cuya confirmación es necesaria para cumplir `<cl>`.
    /// * `<reasonmap>` es un "mapa" de endpoints a códigos de razón de error. Esto mapea los endpoints de los nodos réplica que fallaron al ejecutar la request a un código representando la razón del error. La forma del mapa es empezando con un [i32] n seguido por n pares de <endpoint><failurecode> donde <endpoint> es un [IpAddr](std::net::IpAddr) y <failurecode> es un [u16].
    /// * `<writeType>` es un [String] que representa el tipo de escritura que se estaba intentando realizar. El valor puede ser:
    ///     * "SIMPLE": La escritura no fue de tipo batch ni de tipo counter.
    ///     * "BATCH": La escritura fue de tipo batch (logged). Esto signifca que el log del batch fue escrito correctamente, caso contrario, se debería haber enviado el tipo "BATCH_LOG".
    ///     * "UNLOGGED_BATCH": La escritura fue de tipo batch (unlogged). No hubo intento de escritura en el log del batch.
    ///     * "COUNTER": La escritura fue de tipo counter (batch o no).
    ///     * "BATCH_LOG": El timeout ocurrió durante la escritura en el log del batch cuando una escritura de batch (logged) fue pedida.
    ///     * "CAS": El timeout ocurrió durante el _Compare And Set write/update_ (escritura/actualización).
    ///     * "VIEW": El timeout ocurrió durante una escritura que involucra una actualización de VIEW (vista) y falló en adquirir el lock de vista local (MV) para la clave dentro del timeout.
    ///     * "CDC": El timeout ocurrió cuando la cantidad total de espacio en disco (en MB) que se puede utilizar para almacenar los logs de CDC (Change Data Capture) fue excedida cuando se intentaba escribir en dicho logs.
    ///
    /// TODO: _Quizás meter writeType en un enum._
    WriteFailure(
        String,
        Consistency,
        i32,
        i32,
        HashMap<IpAddr, u16>,
        String,
    ),

    /// _En la documentación del protocolo de Cassandra figura como TODO_.
    CDCWriteFailure(String),

    /// Una excepción ocurrida debido a una operación _Compare And Set write/update_ en contención. La operación CAS fue completada solo parcialmente y la operación puede o no ser completada por la escritura CAS contenedora o la lectura SERIAL/LOCAL_SERIAL.
    ///
    /// El resto del mensaje es `<cl><received><blockfor>`, donde:
    /// * `<cl>` es el nivel de [Consistency](crate::cassandra::notations::consistency::Consistency) de la query que lanzó esta excepción.
    /// * `<received>` es un número ([i32]) que representa la cantidad de nodos que han reconocido la request.
    /// * `<blockfor>` es un número ([i32]) que representa la cantidad de réplicas cuya confirmación es necesaria para cumplir `<cl>`.
    CASWriteUnknown(String, Consistency, i32, i32),

    /// La query enviada tiene un error de sintaxis.
    SyntaxError(String),

    /// El usuario logueado no tiene los permisos necesarios para realizar la query.
    Unauthorized(String),

    /// La query es sintácticamente correcta pero inválida.
    Invalid(String),

    /// La query es inválida debido a algún problema de configuración.
    ConfigError(String),

    /// La query intentó crear un _keyspace_ o una tabla que ya existía.
    ///
    /// El resto del mensaje es `<ks><table>`, donde:
    /// * `<ks>` es un [String] representando el _keyspace_ que ya existía, o el _keyspace_ al que pertenece la tabla que ya existía.
    /// * `<table>` es un [String] representando el nombre de la tabla que ya existía. Si la query intentó crear un _keyspace_, <table> estará presente pero será el string vacío.
    AlreadyExists(String, String, String),

    /// Puede ser lanzado mientras una expresión preparada intenta ser ejecutada si el ID de la misma no es conocido por este host.
    ///
    /// El resto del mensaje es `<id>`, `id` siendo un número ([u8]) representando el ID desconocido.
    Unprepared(String, u8),
}
