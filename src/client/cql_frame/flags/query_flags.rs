/// Flags específicas para queries CQL
#[derive(Clone, Copy)]
pub enum QueryFlags {
    /// Para vincular valores a la query
    Values = 0x01,
    /// Si se quiere saltar los metadatos en la respuesta
    SkipMetadata = 0x02,
    /// Tamaño deseado de la página si se setea
    PageSize = 0x04,
    /// Estado de paginación
    WithPagingState = 0x08,
    /// Consistencia serial para actualizaciones de datos condicionales
    WithSerialConsistency = 0x10,
    /// Timestamp por defecto (en microsegundos)
    WithDefaultTimestamp = 0x20,
    /// Solo tiene sentido si se usa `Values`, para tener nombres de columnas en los valores
    WithNamesForValues = 0x40,
    /// Keyspace donde debe ejecutarse la query
    WithKeyspace = 0x80,
    /// Tiempo actual en segundos
    WithNowInSeconds = 0x100,
}
