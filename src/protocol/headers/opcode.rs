//! Módulo para el opcode del mensaje el protocolo.

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;

/// Describe la operación a utilizar en el protocolo.
#[derive(PartialEq)]
pub enum Opcode {
    /// Indica una _Response_ en la que ocurrió algún tipo de error procesando una _Request_.
    RequestError,

    /// Indica una _Request_ para pedir inicializar una conexión.
    ///
    /// En cuyo caso, el servidor responderá con [READY](crate::protocol::headers::opcode::Opcode::Ready)
    /// o [AUTHENTICATE](crate::protocol::headers::opcode::Opcode::Authenticate).
    Startup,

    /// Indica una _Response_ en la que el servidor enuncia que está listo para recibir _queries_.
    ///
    /// Esto normalmente ocurre si la conexión no requiere de autenticación.
    Ready,

    /// Indica una _Response_ en la que el servidor pide credenciales al cliente para autorizar
    /// la conexión. El mecanismo de autenticación en sí es especificado por el servidor en este
    /// tipo de mensaje.
    ///
    /// Consiste en una serie de [AUTH_CHALLENGE](crate::protocol::headers::opcode::Opcode::AuthChallenge)S,
    /// seguidos de [AUTH_RESPONSE](crate::protocol::headers::opcode::Opcode::AuthResponse)S. Los detalles
    /// dependen del autenticador específico en uso.
    ///
    /// El intercambio termina con un [AUTH_SUCCESS](crate::protocol::headers::opcode::Opcode::AuthSuccess)
    /// desde el servidor, o un [ERROR](crate::protocol::headers::opcode::Opcode::RequestError).
    ///
    /// Este mensaje se manda como respuesta a una _Request_ [STARTUP](crate::protocol::headers::opcode::Opcode::Startup),
    /// pero autenticación es requerida, en cuyo caso el cliente ha de responder con [AUTH_RESPONSE](crate::protocol::headers::opcode::Opcode::AuthResponse).
    Authenticate,

    /// Indica una _Request_ en la que el cliente le pregunta al servidor qué tipo de opciones
    /// hay para un [STARTUP](crate::protocol::headers::opcode::Opcode::Startup).
    ///
    /// El servidor responderá con una _Response_ de tipo [SUPPORTED](crate::protocol::headers::opcode::Opcode::Supported).
    Options,

    /// Indica una _Response_ en consecuencia a una _Request_ [OPTIONS](crate::protocol::headers::opcode::Opcode::Options)
    /// del cliente, e indica las opciones disponibles para un potencial [STARTUP](crate::protocol::headers::opcode::Opcode::Startup).
    ///
    /// Dichas opciones y sus valores podrán ser expresados con un [multimap](crate::protocol::aliases::types::SupportedMultiMap)
    /// correspondiente.
    Supported,

    /// Indica una _Request_ en la que el cliente hace una _query_ al servidor.
    ///
    /// Dicha _query_ estará serializada con el formato `<query><query_parameters>` donde:
    /// * `<query>` será un [String] conteniendo la _query_ misma.
    /// * ` <query_parameters>` tendrá el tipo `<consistency><flags>[<n>[name_1]<value_1>...[name_n]<value_n>][<result_page_size>][<paging_state>][<serial_consistency>][<timestamp>][<keyspace>][<now_in_seconds>]` donde:
    ///     - `<consistency>` es un dato de tipo [Consistency](crate::protocol::notations::consistency::Consistency).
    ///     - `<flags>` es un [Int](crate::protocol::aliases::types::Int) donde sus bits representan
    ///       datos de tipo [QueryFlag](crate::protocol::messages::requests::query_flags::QueryFlag).
    ///
    /// <div class="warning">
    ///
    /// El parámetro de [Consistency](crate::protocol::notations::consistency::Consistency) podría
    /// ser ignorado según cada _query_ necesite.
    ///
    /// </div>
    ///
    /// En un caso exitoso, el servidor responderá con una _Response_ de tipo [RESULT](crate::protocol::headers::opcode::Opcode::Result).
    Query,

    /// Indica una _Response_ a una _query_ (esto es, a un mensaje de tipo [QUERY](crate::protocol::headers::opcode::Opcode::Query),
    /// [PREPARE](crate::protocol::headers::opcode::Opcode::Prepare), [EXECUTE](crate::protocol::headers::opcode::Opcode::Execute)
    /// o [BATCH](crate::protocol::headers::opcode::Opcode::Batch)).
    ///
    /// El contenido del mensaje vendrá acompañado de un [Int](crate::protocol::aliases::types::Int)
    /// indicando el [tipo](crate::protocol::messages::responses::result_kinds::ResultKind) de resultado.
    /// El contenido a continuación depende de dicho tipo.
    Result,

    /// Indica una _Request_ en la que el cliente prepara una _query_ para su posterior
    /// [ejecución](crate::protocol::headers::opcode::Opcode::Execute).
    ///
    /// El contenido tendrá el formato `<query><flags>[<keyspace>]`, donde:
    /// * `<query>` es un [String] representando la _query_ misma.
    /// * `<flags>` es un [Int](crate::protocol::aliases::types::Int) cuyos bits corresponden a datos
    ///   de tipo [PrepareFlag](crate::protocol::messages::requests::query_flags::QueryFlag).
    ///
    /// En caso de éxito, el servidor responderá con un mensaje [RESULT](crate::protocol::headers::opcode::Opcode::Result)
    /// de tipo [Prepared](crate::protocol::messages::responses::result_kinds::ResultKind::Prepared).
    Prepare,

    /// Indica una _request_ en la que el cliente pide ejecutar una _query_ [preparada](crate::protocol::headers::opcode::Opcode::Prepare).
    ///
    /// El contenido tendrá el formato `<id><result_metadata_id><query_parameters>` donde:
    /// * `<id>` será un vector de [Byte]s indicando el ID de la _query_ preparada.
    ///   Debería ser el mismo ID que el devuelto por una respuesta a una _Request_ [PREPARE](crate::protocol::headers::opcode::Opcode::Prepare).
    /// * `<result_metadata_id>` es un set de metadatos de cambios aplicados, enviados en conjunto
    ///   en una _Response_ [Result](crate::protocol::headers::opcode::Opcode::Result)/[Rows](crate::protocol::messages::responses::result_kinds::ResultKind::Rows).
    /// * `<query_parameters>` tiene la misma definición que [QUERY](crate::protocol::headers::opcode::Opcode::Query).
    Execute,

    /// Indica una _Request_ para registrar esta conexión para "escuchar" [eventos](crate::protocol::headers::opcode::Opcode::Event).
    ///
    /// El servidor responderá con una _Response_ [READY](crate::protocol::headers::opcode::Opcode::Ready).
    ///
    /// <div class="warning">
    ///
    /// Si un cliente mantiene múltiples conexiones a un nodo y/o conexiones a múltiples nodos,
    /// es recomendable dedicar algunas de estas conexiones a escuchar los eventos. <br/>
    /// Sin embargo, **no se debería registrar para eventos en todas las conexiones,** pues eso
    /// conllevaría a recibir los mismos mensajes de eventos múltiples veces.
    ///
    /// </div>
    Register,

    /// Indica una _Response_ en la que el server comunica un [evento](crate::protocol::messages::responses::events::event_types::EventType).
    ///
    /// Un cliente sólo escuchará eventos a los que se ha [registrado](crate::protocol::headers::opcode::Opcode::Register).
    ///
    /// Todos los mensajes de evento tendrán un _streamId_ de `-1`.
    Event,

    /// Indica una _Request_ donde se pide ejecutar un conjunto de _queries_
    /// ([preparadas](crate::protocol::headers::opcode::Opcode::Prepare) o no).
    ///
    /// El contenido del mensaje tendrá el formato `<type><n><query_1>...<query_n><consistency><flags>[<serial_consistency>][<timestamp>][<keyspace>][<now_in_seconds>]`, done:
    /// * `<type>` será un [BatchType](crate::protocol::messages::requests::batch_types::BatchType).
    /// * `<flags>` es un [Int](crate::protocol::aliases::types::Int) cuyos bits corresponden a datos
    ///   de tipo [BatchFlag](crate::protocol::messages::requests::batch_flags::BatchFlag).
    ///   Estas flags son similares a los de [QUERY](crate::protocol::headers::opcode::Opcode::Query)
    ///   y [EXECUTE](crate::protocol::headers::opcode::Opcode::Register), excepto que los últimos 4 bits
    ///   son siempre `0` pues sus opciones no tienen sentido en el contexto de un _batch_.
    /// * `<n>` es un [Short](crate::protocol::aliases::types::Short) indicando la cantidad de _queries_ a continuación.
    /// * `<query_1>...<query_n>` son las _queries_ a ejecutar. Cada _query_ `<query_i>` tendrá el
    ///   formato `<kind><string_or_id><n>[<name_1>]<value_1>...[<name_n>]<value_n>` donde:
    ///     - `<kind>` será un [bool] indicando si la _query_ fue preparada o no.
    ///     - `<string_or_id>` depende de `<kind>`.
    ///         * Si `<kind> == false`, `<string_or_id>` es un [String] indicando la _query_ misma.
    ///         * Si `<kind> == true`, `<string_or_id>` es un vector de [Byte]s indicando el ID de la _query_ preparada.
    ///     - `<n>` es un [Short](crate::protocol::aliases::types::Short) indicando la cantidad de valores
    ///       a continuación _(podría ser `0`)_.
    ///     - `<name_i>` es un nombre opcional al valor que sigue. Sólo ha de estar presente si se
    ///       declaró la [flag correspondiente](crate::protocol::messages::requests::batch_flags::BatchFlag::WithNamesForValues).
    ///     - `<value_i>` es un [Value](crate::protocol::notations::value::Value) para la variable `i`.
    Batch,

    /// Indica un "desafío" de autenticación que el cliente debe cumplir para autorizar la conexión.
    ///
    /// El contenido es un conjunto de [Byte]s cuyo significado cambia según el autenticador en uso.
    AuthChallenge,

    /// Indica una respuesta a un [desafío](crate::protocol::headers::opcode::Opcode::AuthChallenge)
    /// de autenticación por parte del servidor.
    ///
    /// El contenido es un conjunto de [Byte]s cuyo significado cambia según el autenticador en uso.
    ///
    /// El servidor responderá con un mensaje [AUTH_CHALLENGE](crate::protocol::headers::opcode::Opcode::AuthChallenge)
    /// para seguir con la autenticación, [AUTH_SUCCESS](crate::protocol::headers::opcode::Opcode::AuthSuccess) si
    /// la autenticación terminó con éxito, o [ERROR](crate::protocol::headers::opcode::Opcode::RequestError) si
    /// ocurrió algún error de por medio.
    AuthResponse,

    /// Indica el éxito de un intercambio de autenticación por parte del servidor.
    ///
    /// El contenido es un conjunto de [Byte]s con información adicional final que el cliente podría
    /// necesitar para finalizar el proceso, dependiendo del autenticador en uso.
    AuthSuccess,
}

impl Byteable for Opcode {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::RequestError => vec![0x0],
            Self::Startup => vec![0x1],
            Self::Ready => vec![0x2],
            Self::Authenticate => vec![0x3],
            Self::Options => vec![0x5],
            Self::Supported => vec![0x6],
            Self::Query => vec![0x7],
            Self::Result => vec![0x8],
            Self::Prepare => vec![0x9],
            Self::Execute => vec![0xA],
            Self::Register => vec![0xB],
            Self::Event => vec![0xC],
            Self::Batch => vec![0xD],
            Self::AuthChallenge => vec![0xE],
            Self::AuthResponse => vec![0xF],
            Self::AuthSuccess => vec![0x10],
        }
    }
}

impl TryFrom<Byte> for Opcode {
    type Error = Error;
    fn try_from(byte: Byte) -> Result<Self, Self::Error> {
        match byte {
            0x00 => Ok(Opcode::RequestError),
            0x01 => Ok(Opcode::Startup),
            0x02 => Ok(Opcode::Ready),
            0x03 => Ok(Opcode::Authenticate),
            0x05 => Ok(Opcode::Options),
            0x06 => Ok(Opcode::Supported),
            0x07 => Ok(Opcode::Query),
            0x08 => Ok(Opcode::Result),
            0x09 => Ok(Opcode::Prepare),
            0x0A => Ok(Opcode::Execute),
            0x0B => Ok(Opcode::Register),
            0x0C => Ok(Opcode::Event),
            0x0D => Ok(Opcode::Batch),
            0x0E => Ok(Opcode::AuthChallenge),
            0x0F => Ok(Opcode::AuthResponse),
            0x10 => Ok(Opcode::AuthSuccess),
            _ => Err(Error::ConfigError(
                "El opcode recibido no es valido".to_string(),
            )), // TODO: Ver que mandar en el mensaje
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::errors::error::Error;
    use crate::protocol::headers::opcode::Opcode;
    use crate::protocol::traits::Byteable;

    #[test]
    fn test_1_serializar() {
        let opcodes = [
            Opcode::Startup,
            Opcode::Prepare,
            Opcode::Result,
            Opcode::Event,
            Opcode::AuthChallenge,
        ];
        let opcodes_serials = [0x1, 0x9, 0x8, 0xC, 0xE];
        let opcodes_len = opcodes.len();

        for i in 0..opcodes_len {
            let serialized = opcodes[i].as_bytes();

            assert_eq!(serialized.len(), 1);
            assert_eq!(serialized[0], opcodes_serials[i]);
        }
    }

    #[test]
    fn test_2_deserializar() {
        let opcode_res = Opcode::try_from(0x10);

        assert!(opcode_res.is_ok());
        if let Ok(opcode) = opcode_res {
            assert!(matches!(opcode, Opcode::AuthSuccess));
        }
    }

    #[test]
    fn test_3_id_incorrecto() {
        let muy_mal = Opcode::try_from(0x4); // el 0x4 no tiene un opcode asignado

        assert!(muy_mal.is_err());
        if let Err(err) = muy_mal {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
