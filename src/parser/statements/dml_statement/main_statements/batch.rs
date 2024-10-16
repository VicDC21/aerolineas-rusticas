use crate::parser::statements::dml_statement::dml_statement_parser::DmlStatement;

/// [ UNLOGGED | COUNTER ] el tipo de BATCH por defecto es LOGGED
#[derive(Default)]
pub enum BatchType {
    #[default]
    /// Batch en estado default, asegura que eventualmente se completen todas las operaciones (o ninguna lo hara).
    Logged,
    /// Si se usa la opcion UNLOGGED, un batch fallido puede dejar el 'patch' solo parcialmente aplicado
    Unlogged,
    /// Se usa la opcion COUNTER para batched counter updates.
    Counter,
}

/// batch_statement ::=  BEGIN [ UNLOGGED | COUNTER ] BATCH
///                      modification_statement ( ';' modification_statement )*
///                      APPLY BATCH
pub struct Batch {
    /// Tipo de batch indicado mas especificamente en el tipo de dato correspondiente [BatchType].
    pub batch_type: BatchType,

    /// Consultas de batch realizables.
    pub queries: Vec<DmlStatement>,
}

impl Batch {
    fn new(batch_type: BatchType, queries: Vec<DmlStatement>) -> Batch {
        Batch {
            batch_type,
            queries,
        }
    }
}

/// Builder del struct Batch
#[derive(Default)]
pub struct BatchBuilder {
    batch_type: BatchType,
    queries: Vec<DmlStatement>,
}

impl BatchBuilder {
    /// Setea el tipo de batch.
    pub fn set_batch_clause(&mut self, batch_type: BatchType) {
        self.batch_type = batch_type;
    }

    /// Setea las queries realizables.
    pub fn set_queries(&mut self, queries: Vec<DmlStatement>) {
        self.queries = queries;
    }

    /// Construye el struct [Batch] con los previamente datos almacenados.
    pub fn build(self) -> Batch {
        Batch::new(self.batch_type, self.queries)
    }
}
