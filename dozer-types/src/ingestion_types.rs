use prettytable::Table;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    errors::internal::BoxedError,
    types::{Commit, OperationEvent},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IngestionOperation {
    OperationEvent(OperationEvent),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum IngestionMessage {
    Begin(),
    OperationEvent(OperationEvent),
    Commit(Commit),
}

#[derive(Error, Debug)]
pub enum IngestorError {
    #[error("Failed to send message on channel")]
    ChannelError(#[from] BoxedError),
}

pub trait IngestorForwarder: Send + Sync + Debug {
    fn forward(&self, msg: ((u64, u64), IngestionOperation)) -> Result<(), IngestorError>;
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, ::prost::Message, Hash)]
pub struct EthFilter {
    // Starting block
    #[prost(uint64, optional, tag = "1")]
    pub from_block: Option<u64>,
    #[prost(uint64, optional, tag = "2")]
    pub to_block: Option<u64>,
    #[prost(string, repeated, tag = "3")]
    #[serde(default)]
    pub addresses: Vec<String>,
    #[prost(string, repeated, tag = "4")]
    #[serde(default)]
    pub topics: Vec<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, ::prost::Message, Hash)]
pub struct EthConfig {
    #[prost(message, optional, tag = "1")]
    pub filter: Option<EthFilter>,
    #[prost(string, tag = "2")]
    pub wss_url: String,
    #[prost(message, repeated, tag = "3")]
    #[serde(default)]
    pub contracts: Vec<EthContract>,
}

impl EthConfig {
    pub fn convert_to_table(&self) -> Table {
        let mut table = table!(["wss_url", self.wss_url]);

        if let Some(filter) = &self.filter {
            let mut addresses_table = table!();
            for address in &filter.addresses {
                addresses_table.add_row(row![address]);
            }

            let mut topics_table = table!();
            for topic in &filter.topics {
                topics_table.add_row(row![topic]);
            }

            let filter_table = table!(
                [
                    "from_block",
                    filter
                        .from_block
                        .map_or("-------".to_string(), |f| f.to_string())
                ],
                [
                    "to_block",
                    filter
                        .to_block
                        .map_or("-------".to_string(), |f| f.to_string())
                ],
                ["addresses", addresses_table],
                ["topics", topics_table]
            );
            table.add_row(row!["filter", filter_table]);
        }

        table
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, ::prost::Message, Hash)]
pub struct EthContract {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub address: String,
    #[prost(string, tag = "3")]
    pub abi: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, ::prost::Message, Hash)]
pub struct KafkaConfig {
    #[prost(string, tag = "1")]
    pub broker: String,
    #[prost(string, optional, tag = "3")]
    pub schema_registry_url: Option<String>,
}

impl KafkaConfig {
    pub fn convert_to_table(&self) -> Table {
        table!(
            ["broker", self.broker],
            [
                "schema registry url",
                self.schema_registry_url
                    .as_ref()
                    .map_or("--------", |url| url)
            ]
        )
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, ::prost::Message, Hash)]
pub struct SnowflakeConfig {
    #[prost(string, tag = "1")]
    pub server: String,
    #[prost(string, tag = "2")]
    pub port: String,
    #[prost(string, tag = "3")]
    pub user: String,
    #[prost(string, tag = "4")]
    pub password: String,
    #[prost(string, tag = "5")]
    pub database: String,
    #[prost(string, tag = "6")]
    pub schema: String,
    #[prost(string, tag = "7")]
    pub warehouse: String,
    #[prost(string, optional, tag = "8")]
    pub driver: Option<String>,
}

impl SnowflakeConfig {
    pub fn convert_to_table(&self) -> Table {
        table!(
            ["server", self.server],
            ["port", self.port],
            ["user", self.user],
            ["password", "************"],
            ["database", self.database],
            ["schema", self.schema],
            ["warehouse", self.warehouse],
            ["driver", self.driver.as_ref().map_or("default", |d| d)]
        )
    }
}
