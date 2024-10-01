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
    AuthSuccess
}


impl Byteable for Opcode {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Error => &[0],
            Self::Startup => &[1],
            Self::Ready => &[2],
            Self::Authenticate => &[3],
            Self::Options => &[5],
            Self::Supported => &[6],
            Self::Query => &[7],
            Self::Result => &[8],
            Self::Prepare => &[9],
            Self::Execute => &[10],
            Self::Register => &[11],
            Self::Event => &[12],
            Self::Batch => &[13],
            Self::AuthChallenge => &[14],
            Self::AuthResponse => &[15],
            Self::AuthSuccess => &[16]
        }
    }
}