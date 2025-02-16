//! Traits en común con objetos del protocolo de Cassandra.

use {
    crate::aliases::types::Byte,
    std::ops::{BitAnd, BitOrAssign},
};

/// Colapsa una propiedad en una colección de bytes.
pub trait Byteable {
    /// Transforma el objeto en un vector de bytes.
    fn as_bytes(&self) -> Vec<Byte>;
}

/// Une muchas propiedades (pensadas como máscaras) en un sólo número.
pub trait Maskable<T: BitOrAssign + BitAnd<Output = T> + Copy + PartialEq> {
    /// Devuelve un acumulador del tipo T para las máscaras.
    fn base_mask() -> T;

    /// Convierte una propiedad en un número binario.
    fn collapse(&self) -> T;

    /// Comprueba si el elemento acumulado tiene los bits especificados.
    fn has_bits(base: T, mask: T) -> bool {
        base & mask == mask
    }

    /// Verifica si en una colección de máscaras se encuentra una en particular.
    fn has_mask(masks: &T, mask: &Self) -> bool {
        Self::has_bits(masks.to_owned(), mask.collapse())
    }

    /// Une todas las máscaras.
    fn accumulate(masks: &[&Self]) -> T {
        let mut accumulator = Self::base_mask();
        for msk in masks {
            accumulator |= msk.collapse();
        }
        accumulator
    }
}
