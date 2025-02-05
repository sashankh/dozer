use crate::connectors::postgres::xlog_mapper::TableColumn;
use crate::errors::PostgresSchemaError::{
    ColumnTypeNotFound, ColumnTypeNotSupported, CustomTypeNotSupported, ValueConversionError,
};
use crate::errors::{ConnectorError, PostgresSchemaError};
use dozer_types::bytes::Bytes;
use dozer_types::chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Offset, Utc};
use dozer_types::ordered_float::OrderedFloat;
use dozer_types::{rust_decimal, types::*};
use postgres::{Column, Row};
use postgres_types::{Type, WasNull};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::error::Error;
use std::vec;

pub fn postgres_type_to_field(
    value: Option<&Bytes>,
    column: &TableColumn,
) -> Result<Field, PostgresSchemaError> {
    value.map_or(Ok(Field::Null), |v| {
        column
            .r#type
            .clone()
            .map_or(Err(ColumnTypeNotFound), |column_type| match column_type {
                Type::INT2 | Type::INT4 | Type::INT8 => Ok(Field::Int(
                    String::from_utf8(v.to_vec()).unwrap().parse().unwrap(),
                )),
                Type::FLOAT4 | Type::FLOAT8 => Ok(Field::Float(OrderedFloat(
                    String::from_utf8(v.to_vec())
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                ))),
                Type::TEXT | Type::VARCHAR | Type::CHAR => {
                    Ok(Field::String(String::from_utf8(v.to_vec()).unwrap()))
                }
                Type::BYTEA => Ok(Field::Binary(v.to_vec())),
                Type::NUMERIC => Ok(Field::Decimal(
                    Decimal::from_f64(
                        String::from_utf8(v.to_vec())
                            .unwrap()
                            .parse::<f64>()
                            .unwrap(),
                    )
                    .unwrap(),
                )),
                Type::TIMESTAMP => {
                    let date = NaiveDateTime::parse_from_str(
                        String::from_utf8(v.to_vec()).unwrap().as_str(),
                        "%Y-%m-%d %H:%M:%S",
                    )
                    .unwrap();
                    Ok(Field::Timestamp(DateTime::from_utc(date, Utc.fix())))
                }
                Type::TIMESTAMPTZ => {
                    let date: DateTime<FixedOffset> = DateTime::parse_from_str(
                        String::from_utf8(v.to_vec()).unwrap().as_str(),
                        "%Y-%m-%d %H:%M:%S%.f%#z",
                    )
                    .unwrap();
                    Ok(Field::Timestamp(date))
                }
                Type::DATE => {
                    let date: NaiveDate = NaiveDate::parse_from_str(
                        String::from_utf8(v.to_vec()).unwrap().as_str(),
                        DATE_FORMAT,
                    )
                    .unwrap();
                    Ok(Field::from(date))
                }
                Type::JSONB | Type::JSON => Ok(Field::Bson(v.to_vec())),
                Type::BOOL => Ok(Field::Boolean(v.slice(0..1) == "t")),
                _ => Err(ColumnTypeNotSupported(column_type.name().to_string())),
            })
    })
}

pub fn postgres_type_to_dozer_type(column_type: Type) -> Result<FieldType, PostgresSchemaError> {
    match column_type {
        Type::BOOL => Ok(FieldType::Boolean),
        Type::INT2 | Type::INT4 | Type::INT8 => Ok(FieldType::Int),
        Type::CHAR | Type::TEXT | Type::VARCHAR => Ok(FieldType::String),
        Type::FLOAT4 | Type::FLOAT8 => Ok(FieldType::Float),
        Type::BIT => Ok(FieldType::Binary),
        Type::TIMESTAMP | Type::TIMESTAMPTZ => Ok(FieldType::Timestamp),
        Type::NUMERIC => Ok(FieldType::Decimal),
        Type::JSONB => Ok(FieldType::Bson),
        Type::DATE => Ok(FieldType::Date),
        _ => Err(ColumnTypeNotSupported(column_type.name().to_string())),
    }
}

fn handle_error(e: postgres::error::Error) -> Result<Field, PostgresSchemaError> {
    if let Some(e) = e.source() {
        if let Some(_e) = e.downcast_ref::<WasNull>() {
            Ok(Field::Null)
        } else {
            Err(ValueConversionError(e.to_string()))
        }
    } else {
        Err(ValueConversionError(e.to_string()))
    }
}

macro_rules! convert_row_value_to_field {
    ($a:ident, $b:ident, $c:ty) => {{
        let value: Result<$c, _> = $a.try_get($b);
        value.map_or_else(handle_error, |val| Ok(Field::from(val)))
    }};
}

pub fn value_to_field(
    row: &Row,
    idx: usize,
    col_type: &Type,
) -> Result<Field, PostgresSchemaError> {
    match col_type {
        &Type::BOOL => convert_row_value_to_field!(row, idx, bool),
        &Type::INT2 => convert_row_value_to_field!(row, idx, i16),
        &Type::INT4 => convert_row_value_to_field!(row, idx, i32),
        &Type::INT8 => convert_row_value_to_field!(row, idx, i64),
        &Type::CHAR | &Type::TEXT | &Type::VARCHAR | &Type::BPCHAR => {
            convert_row_value_to_field!(row, idx, String)
        }
        &Type::FLOAT4 => convert_row_value_to_field!(row, idx, f32),
        &Type::FLOAT8 => convert_row_value_to_field!(row, idx, f64),
        &Type::TIMESTAMP => convert_row_value_to_field!(row, idx, NaiveDateTime),
        &Type::TIMESTAMPTZ => convert_row_value_to_field!(row, idx, DateTime<FixedOffset>),
        &Type::NUMERIC => convert_row_value_to_field!(row, idx, Decimal),
        &Type::DATE => convert_row_value_to_field!(row, idx, NaiveDate),
        &Type::BYTEA => {
            let value: Result<Vec<u8>, _> = row.try_get(idx);
            value.map_or_else(handle_error, |v| Ok(Field::Binary(v)))
        }
        &Type::JSONB => {
            let value: Result<Vec<u8>, _> = row.try_get(idx);
            value.map_or_else(handle_error, |v| Ok(Field::Bson(v)))
        }
        _ => {
            if col_type.schema() == "pg_catalog" {
                Err(ColumnTypeNotSupported(col_type.name().to_string()))
            } else {
                Err(CustomTypeNotSupported)
            }
        }
    }
}

pub fn get_values(row: &Row, columns: &[Column]) -> Result<Vec<Field>, PostgresSchemaError> {
    let mut values: Vec<Field> = vec![];
    for (idx, col) in columns.iter().enumerate() {
        let val = value_to_field(row, idx, col.type_());
        match val {
            Ok(val) => values.push(val),
            Err(e) => return Err(e),
        };
    }
    Ok(values)
}

pub fn map_row_to_operation_event(
    _table_name: String,
    identifier: SchemaIdentifier,
    row: &Row,
    columns: &[Column],
    seq_no: u64,
) -> Result<OperationEvent, PostgresSchemaError> {
    match get_values(row, columns) {
        Ok(values) => Ok(OperationEvent {
            operation: Operation::Insert {
                new: Record::new(Some(identifier), values, None),
            },
            seq_no,
        }),
        Err(e) => Err(e),
    }
}

pub fn map_schema(rel_id: &u32, columns: &[Column]) -> Result<Schema, ConnectorError> {
    let field_defs: Result<Vec<FieldDefinition>, _> =
        columns.iter().map(convert_column_to_field).collect();

    Ok(Schema {
        identifier: Some(SchemaIdentifier {
            id: *rel_id,
            version: 1,
        }),
        fields: field_defs.unwrap(),
        primary_index: vec![0],
    })
}

pub fn convert_column_to_field(column: &Column) -> Result<FieldDefinition, PostgresSchemaError> {
    postgres_type_to_dozer_type(column.type_().clone()).map(|typ| FieldDefinition {
        name: column.name().to_string(),
        typ,
        nullable: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dozer_types::chrono::NaiveDate;

    #[macro_export]
    macro_rules! test_conversion {
        ($a:expr,$b:expr,$c:expr) => {
            let value = postgres_type_to_field(
                Some(&Bytes::from($a)),
                &TableColumn {
                    name: "column".to_string(),
                    type_id: $b.oid() as i32,
                    flags: 0,
                    r#type: Some($b),
                    idx: 0,
                },
            );
            assert_eq!(value.unwrap(), $c);
        };
    }

    #[test]
    fn it_converts_postgres_type_to_field() {
        test_conversion!("12", Type::INT8, Field::Int(12));
        test_conversion!("4.7809", Type::FLOAT8, Field::Float(OrderedFloat(4.7809)));
        let value = String::from("Test text");
        test_conversion!("Test text", Type::TEXT, Field::String(value));

        // UTF-8 bytes representation of json (https://www.charset.org/utf-8)
        let value: Vec<u8> = vec![98, 121, 116, 101, 97];
        test_conversion!("bytea", Type::BYTEA, Field::Binary(value));

        let value = Decimal::from_f64(8.28).unwrap();
        test_conversion!("8.28", Type::NUMERIC, Field::Decimal(value));

        let value = DateTime::from_utc(
            NaiveDate::from_ymd(2022, 9, 16).and_hms(5, 56, 29),
            Utc.fix(),
        );
        test_conversion!(
            "2022-09-16 05:56:29",
            Type::TIMESTAMP,
            Field::Timestamp(value)
        );

        let value = DateTime::from_utc(
            NaiveDate::from_ymd(2022, 9, 16).and_hms_micro(3, 56, 30, 959787),
            Utc.fix(),
        );
        test_conversion!(
            "2022-09-16 10:56:30.959787+07",
            Type::TIMESTAMPTZ,
            Field::Timestamp(value)
        );

        // UTF-8 bytes representation of json (https://www.charset.org/utf-8)
        let value = vec![123, 34, 97, 98, 99, 34, 58, 34, 102, 111, 111, 34, 125];
        test_conversion!("{\"abc\":\"foo\"}", Type::JSONB, Field::Bson(value));

        test_conversion!("t", Type::BOOL, Field::Boolean(true));
        test_conversion!("f", Type::BOOL, Field::Boolean(false));
    }

    #[test]
    fn test_none_value() {
        let value = postgres_type_to_field(
            None,
            &TableColumn {
                name: "column".to_string(),
                type_id: Type::VARCHAR.oid() as i32,
                flags: 0,
                r#type: Some(Type::VARCHAR),
                idx: 0,
            },
        );
        assert_eq!(value.unwrap(), Field::Null);
    }
}
