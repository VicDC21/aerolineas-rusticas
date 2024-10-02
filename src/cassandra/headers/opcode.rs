//! Módulo para el opcode del mensaje el protocolo.

use crate::cassandra::traits::Byteable;

/// Describe la operación a utilizar en el protocolo.
///
/// TODO: [Mejores descripciones]
pub enum Opcode {
    /// Error Variant
    Error,

    /// Startup Variant
    Startup,

    /// Ready Variant
    Ready,

    /// Authenticate Variant
    Authenticate,

    /// Options Variant
    Options,

    /// Supported Variant
    Supported,

    /// Query Variant
    Query,

    /// Result Variant
    Result,

    /// Prepare Variant
    Prepare,

    /// Execute Variant
    Execute,

    /// Register Variant
    Register,

    /// Event Variant
    Event,

    /// Batch Variant
    Batch,

    /// AuthChallenge Variant
    AuthChallenge,

    /// AuthResponse Variant
    AuthResponse,

    /// AuthSuccess Variant
    AuthSuccess,
}

impl Byteable for Opcode {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Error => vec![0],
            Self::Startup => vec![1],
            Self::Ready => vec![2],
            Self::Authenticate => vec![3],
            Self::Options => vec![5],
            Self::Supported => vec![6],
            Self::Query => vec![7],
            Self::Result => vec![8],
            Self::Prepare => vec![9],
            Self::Execute => vec![10],
            Self::Register => vec![11],
            Self::Event => vec![12],
            Self::Batch => vec![13],
            Self::AuthChallenge => vec![14],
            Self::AuthResponse => vec![15],
            Self::AuthSuccess => vec![16],
        }
    }
}
