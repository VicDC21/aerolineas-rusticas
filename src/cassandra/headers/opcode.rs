//! Módulo para el opcode del mensaje el protocolo.

use crate::cassandra::aliases::types::Byte;
use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::Byteable;

/// Describe la operación a utilizar en el protocolo.
///
/// TODO: [Mejores descripciones]
pub enum Opcode {
    /// Error Variant
    OpCodeError,

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
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::OpCodeError => vec![0x0],
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
            0x00 => Ok(Opcode::OpCodeError),
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
            )), // Todo: ver que mandar en el mensaje
        }
    }
}
