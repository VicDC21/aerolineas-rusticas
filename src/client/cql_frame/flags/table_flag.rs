/// Flags individuales para tablas
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TableFlag {
    /// Flag para ordenamiento
    ClusteringOrder = 0x01,
    /// Flags para compresión
    Compression = 0x02,
    /// Flags para caching
    Caching = 0x04,
    /// Flags para compaction
    Compaction = 0x08,
}
