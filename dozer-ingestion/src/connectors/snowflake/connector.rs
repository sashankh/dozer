#[cfg(feature = "snowflake")]
use odbc::create_environment_v3;
use std::sync::Arc;
#[cfg(feature = "snowflake")]
use std::time::Duration;

#[cfg(feature = "snowflake")]
use crate::connectors::snowflake::connection::client::Client;
use crate::connectors::{Connector, ValidationResults};
use crate::ingestion::Ingestor;
use crate::{connectors::TableInfo, errors::ConnectorError};
use dozer_types::ingestion_types::SnowflakeConfig;
use dozer_types::parking_lot::RwLock;

#[cfg(feature = "snowflake")]
use crate::connectors::snowflake::snapshotter::Snapshotter;
#[cfg(feature = "snowflake")]
use crate::connectors::snowflake::stream_consumer::StreamConsumer;
#[cfg(feature = "snowflake")]
use crate::errors::SnowflakeError::ConnectionError;
use dozer_types::models::source::Source;
use dozer_types::types::SchemaWithChangesType;
use tokio::runtime::Runtime;
#[cfg(feature = "snowflake")]
use tokio::time;

pub struct SnowflakeConnector {
    pub id: u64,
    config: SnowflakeConfig,
    ingestor: Option<Arc<RwLock<Ingestor>>>,
    tables: Option<Vec<TableInfo>>,
}

impl SnowflakeConnector {
    pub fn new(id: u64, config: SnowflakeConfig) -> Self {
        Self {
            id,
            config,
            ingestor: None,
            tables: None,
        }
    }
}

impl Connector for SnowflakeConnector {
    fn get_connection_groups(sources: Vec<Source>) -> Vec<Vec<Source>> {
        sources.iter().map(|s| vec![s.clone()]).collect()
    }

    #[cfg(feature = "snowflake")]
    fn get_schemas(
        &self,
        table_names: Option<Vec<TableInfo>>,
    ) -> Result<Vec<SchemaWithChangesType>, ConnectorError> {
        let client = Client::new(&self.config);
        let env = create_environment_v3().map_err(|e| e.unwrap()).unwrap();
        let conn = env
            .connect_with_connection_string(&client.get_conn_string())
            .map_err(|e| ConnectionError(Box::new(e)))?;

        client
            .fetch_tables(table_names, &self.config, &conn)
            .map_err(ConnectorError::SnowflakeError)
    }

    #[cfg(not(feature = "snowflake"))]
    fn get_schemas(
        &self,
        _table_names: Option<Vec<TableInfo>>,
    ) -> Result<Vec<SchemaWithChangesType>, ConnectorError> {
        todo!()
    }

    fn get_tables(&self) -> Result<Vec<TableInfo>, ConnectorError> {
        todo!()
    }

    fn test_connection(&self) -> Result<(), ConnectorError> {
        todo!()
    }

    fn initialize(
        &mut self,
        ingestor: Arc<RwLock<Ingestor>>,
        tables: Option<Vec<TableInfo>>,
    ) -> Result<(), ConnectorError> {
        self.ingestor = Some(ingestor);
        self.tables = tables;
        Ok(())
    }

    fn start(&self, _from_seq: Option<(u64, u64)>) -> Result<(), ConnectorError> {
        let _connector_id = self.id;
        let ingestor = self
            .ingestor
            .as_ref()
            .map_or(Err(ConnectorError::InitializationError), Ok)?
            .clone();

        Runtime::new()
            .unwrap()
            .block_on(async { run(self.config.clone(), self.tables.clone(), ingestor).await })
    }

    fn stop(&self) {}

    fn validate(&self, _tables: Option<Vec<TableInfo>>) -> Result<(), ConnectorError> {
        Ok(())
    }

    fn validate_schemas(&self, _tables: &[TableInfo]) -> Result<ValidationResults, ConnectorError> {
        todo!()
    }
}

#[cfg(feature = "snowflake")]
async fn run(
    config: SnowflakeConfig,
    tables: Option<Vec<TableInfo>>,
    ingestor: Arc<RwLock<Ingestor>>,
) -> Result<(), ConnectorError> {
    let client = Client::new(&config);

    // SNAPSHOT part - run it when stream table doesnt exist
    match tables {
        None => {}
        Some(tables) => {
            for table in tables.iter() {
                let is_stream_created =
                    StreamConsumer::is_stream_created(&client, table.name.clone())?;
                if !is_stream_created {
                    let ingestor_snapshot = Arc::clone(&ingestor);
                    Snapshotter::run(&client, &ingestor_snapshot, table.name.clone())?;
                    StreamConsumer::create_stream(&client, &table.name)?;
                }
            }

            let stream_client = Client::new(&config);
            let ingestor_stream = Arc::clone(&ingestor);
            let mut interval = time::interval(Duration::from_secs(5));

            let mut consumer = StreamConsumer::new();
            loop {
                for table in tables.iter() {
                    consumer.consume_stream(&stream_client, &table.name, &ingestor_stream)?;

                    interval.tick().await;
                }
            }
        }
    };

    Ok(())
}

#[cfg(not(feature = "snowflake"))]
async fn run(
    _config: SnowflakeConfig,
    _tables: Option<Vec<TableInfo>>,
    _ingestor: Arc<RwLock<Ingestor>>,
) -> Result<(), ConnectorError> {
    Ok(())
}
