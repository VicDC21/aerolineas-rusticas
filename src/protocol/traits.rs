//! Traits en común con objetos del protocolo de Cassandra.

use std::ops::{BitAndAssign, BitOrAssign};

use crate::protocol::aliases::types::Byte;

/// Colapsa una propiedad en una colección de bytes.
pub trait Byteable {
    /// Transforma el objeto en un vector de bytes.
    fn as_bytes(&self) -> Vec<Byte>;
}

/// Une muchas propiedades (pensadas como máscaras) en un sólo número.
pub trait Maskable<T: BitOrAssign + BitAndAssign + Copy + PartialEq> {
    /// Devuelve un acumulador del tipo T para las máscaras.
    fn base_mask() -> T;

    /// Convierte una propiedad en un número binario.
    fn collapse(&self) -> T;

    /// Verifica si en una colección de máscaras se encuentra una en particular.
    fn tiene_mask(masks: &T, mask: &Self) -> bool {
        let mut accumulator = mask.collapse();
        accumulator &= *masks;
        accumulator == mask.collapse()
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
