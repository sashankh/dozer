use std::sync::Arc;

use dozer_types::models::source::Source;
use dozer_types::types::ReplicationChangesTrackingType;
use dozer_types::{ingestion_types::IngestionMessage, parking_lot::RwLock};

use crate::connectors::ValidationResults;
use crate::{
    connectors::{Connector, TableInfo},
    errors::ConnectorError,
    ingestion::Ingestor,
};

pub struct EventsConnector {
    pub id: u64,
    pub name: String,
    ingestor: Option<Arc<RwLock<Ingestor>>>,
}

impl EventsConnector {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            ingestor: None,
        }
    }

    pub fn push(&mut self, msg: IngestionMessage) -> Result<(), ConnectorError> {
        let ingestor = self
            .ingestor
            .as_ref()
            .map_or(Err(ConnectorError::InitializationError), Ok)?;

        ingestor
            .write()
            .handle_message(((0, 0), msg))
            .map_err(ConnectorError::IngestorError)
    }
}

impl Connector for EventsConnector {
    fn get_schemas(
        &self,
        _table_names: Option<Vec<TableInfo>>,
    ) -> Result<
        Vec<(
            String,
            dozer_types::types::Schema,
            ReplicationChangesTrackingType,
        )>,
        ConnectorError,
    > {
        Ok(vec![])
    }

    fn get_tables(&self) -> Result<Vec<TableInfo>, ConnectorError> {
        Ok(vec![])
    }

    fn stop(&self) {}

    fn test_connection(&self) -> Result<(), ConnectorError> {
        Ok(())
    }

    fn initialize(
        &mut self,
        ingestor: Arc<RwLock<Ingestor>>,
        _: Option<Vec<TableInfo>>,
    ) -> Result<(), ConnectorError> {
        self.ingestor = Some(ingestor);
        Ok(())
    }

    fn start(&self, _from_seq: Option<(u64, u64)>) -> Result<(), ConnectorError> {
        Ok(())
    }

    fn validate(&self, _tables: Option<Vec<TableInfo>>) -> Result<(), ConnectorError> {
        Ok(())
    }

    fn get_connection_groups(sources: Vec<Source>) -> Vec<Vec<Source>> {
        vec![sources]
    }

    fn validate_schemas(&self, _tables: &[TableInfo]) -> Result<ValidationResults, ConnectorError> {
        todo!()
    }
}
