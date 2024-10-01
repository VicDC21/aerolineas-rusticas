//! Traits en común con objetos del protocolo de Cassandra.

use std::ops::BitOrAssign;

/// Colapsa una propiedad en una colección de bytes.
pub trait Byteable {
    /// Transforma el objeto en una colección de bytes.
    fn as_bytes(&self) -> &[u8];
}

/// Une muchas propiedades (pensadas como máscaras) en un sólo número.
pub trait Maskable<T: BitOrAssign> {
    /// Devuelve un acumulador del tipo T para las máscaras.
    fn base_mask() -> T;

    /// Convierte una propiedad en un número binario.
    fn collapse(&self) -> T;

    /// Une todas las máscaras.
    fn accumulate(mut accumulator: T, masks: &[&Self]) -> T {
        for msk in masks {
            accumulator |= msk.collapse();
        }
        accumulator
    }
}