use dozer_types::types::{FieldDefinition, IndexDefinition, Schema, SchemaIdentifier};

pub fn schema_0() -> (Schema, Vec<IndexDefinition>) {
    (
        Schema {
            identifier: Some(SchemaIdentifier { id: 0, version: 1 }),
            fields: vec![FieldDefinition {
                name: "foo".to_string(),
                typ: dozer_types::types::FieldType::String,
                nullable: true,
            }],
            primary_index: vec![0],
        },
        vec![IndexDefinition::SortedInverted(vec![0])],
    )
}

pub fn schema_1() -> (Schema, Vec<IndexDefinition>) {
    (
        Schema {
            identifier: Some(SchemaIdentifier { id: 1, version: 1 }),
            fields: vec![
                FieldDefinition {
                    name: "a".to_string(),
                    typ: dozer_types::types::FieldType::Int,
                    nullable: true,
                },
                FieldDefinition {
                    name: "b".to_string(),
                    typ: dozer_types::types::FieldType::String,
                    nullable: true,
                },
                FieldDefinition {
                    name: "c".to_string(),
                    typ: dozer_types::types::FieldType::Int,
                    nullable: true,
                },
            ],
            primary_index: vec![0],
        },
        vec![
            IndexDefinition::SortedInverted(vec![0]),
            IndexDefinition::SortedInverted(vec![1]),
            IndexDefinition::SortedInverted(vec![2]),
            // composite index
            IndexDefinition::SortedInverted(vec![0, 1]),
        ],
    )
}

pub fn schema_full_text_single() -> (Schema, Vec<IndexDefinition>) {
    (
        Schema {
            identifier: Some(SchemaIdentifier { id: 2, version: 1 }),
            fields: vec![FieldDefinition {
                name: "foo".to_string(),
                typ: dozer_types::types::FieldType::String,
                nullable: false,
            }],
            primary_index: vec![0],
        },
        vec![IndexDefinition::FullText(0)],
    )
}

// This is for testing appending only schema, which doesn't need a primary index, for example, eth logs.
pub fn schema_empty_primary_index() -> (Schema, Vec<IndexDefinition>) {
    (
        Schema {
            identifier: Some(SchemaIdentifier { id: 3, version: 1 }),
            fields: vec![FieldDefinition {
                name: "foo".to_string(),
                typ: dozer_types::types::FieldType::String,
                nullable: false,
            }],
            primary_index: vec![],
        },
        vec![IndexDefinition::SortedInverted(vec![0])],
    )
}

pub fn schema_multi_indices() -> (Schema, Vec<IndexDefinition>) {
    (
        Schema {
            identifier: Some(SchemaIdentifier { id: 4, version: 1 }),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    typ: dozer_types::types::FieldType::Int,
                    nullable: false,
                },
                FieldDefinition {
                    name: "text".to_string(),
                    typ: dozer_types::types::FieldType::String,
                    nullable: false,
                },
            ],
            primary_index: vec![0],
        },
        vec![
            IndexDefinition::SortedInverted(vec![0]),
            IndexDefinition::FullText(1),
        ],
    )
}
