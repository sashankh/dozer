use std::collections::HashMap;

use dozer_core::dag::{
    dag::DEFAULT_PORT_HANDLE,
    errors::ExecutionError,
    node::{OutputPortDef, OutputPortType, PortHandle, Processor, ProcessorFactory},
};
use dozer_types::types::{FieldDefinition, Schema};
use sqlparser::ast::SelectItem;

use crate::pipeline::{
    errors::PipelineError,
    expression::{
        builder::{BuilderExpressionType, ExpressionBuilder},
        execution::Expression,
        execution::ExpressionExecutor,
    },
};

use super::processor::ProjectionProcessor;

#[derive(Debug)]
pub struct ProjectionProcessorFactory {
    select: Vec<SelectItem>,
}

impl ProjectionProcessorFactory {
    /// Creates a new [`ProjectionProcessorFactory`].
    pub fn _new(select: Vec<SelectItem>) -> Self {
        Self { select }
    }
}

impl ProcessorFactory for ProjectionProcessorFactory {
    fn get_input_ports(&self) -> Vec<PortHandle> {
        vec![DEFAULT_PORT_HANDLE]
    }

    fn get_output_ports(&self) -> Vec<OutputPortDef> {
        vec![OutputPortDef::new(
            DEFAULT_PORT_HANDLE,
            OutputPortType::Stateless,
        )]
    }

    fn get_output_schema(
        &self,
        _output_port: &PortHandle,
        input_schemas: &HashMap<PortHandle, Schema>,
    ) -> Result<Schema, ExecutionError> {
        let input_schema = input_schemas.get(&DEFAULT_PORT_HANDLE).unwrap();
        match self
            .select
            .iter()
            .map(|item| parse_sql_select_item(item, input_schema))
            .collect::<Result<Vec<(String, Expression)>, PipelineError>>()
        {
            Ok(expressions) => {
                let mut output_schema = Schema::empty();

                for e in expressions.iter() {
                    let field_name = e.0.clone();
                    let field_type =
                        e.1.get_type(input_schema)
                            .map_err(|e| ExecutionError::InternalError(Box::new(e)))?;
                    output_schema.fields.push(FieldDefinition::new(
                        field_name,
                        field_type.return_type,
                        field_type.nullable,
                    ));
                }

                Ok(output_schema)
            }
            Err(error) => Err(ExecutionError::InternalStringError(error.to_string())),
        }
    }

    fn build(
        &self,
        input_schemas: HashMap<PortHandle, Schema>,
        _output_schemas: HashMap<PortHandle, Schema>,
    ) -> Result<Box<dyn Processor>, ExecutionError> {
        let schema = match input_schemas.get(&DEFAULT_PORT_HANDLE) {
            Some(schema) => Ok(schema),
            None => Err(ExecutionError::InternalStringError(
                "Invalid Projection input port".to_string(),
            )),
        }?;

        match self
            .select
            .iter()
            .map(|item| parse_sql_select_item(item, schema))
            .collect::<Result<Vec<(String, Expression)>, PipelineError>>()
        {
            Ok(expressions) => Ok(Box::new(ProjectionProcessor::new(
                schema.clone(),
                expressions,
            ))),
            Err(error) => Err(ExecutionError::InternalStringError(error.to_string())),
        }
    }

    fn prepare(
        &self,
        _input_schemas: HashMap<PortHandle, Schema>,
        _output_schemas: HashMap<PortHandle, Schema>,
    ) -> Result<(), ExecutionError> {
        Ok(())
    }
}

pub(crate) fn parse_sql_select_item(
    sql: &SelectItem,
    schema: &Schema,
) -> Result<(String, Expression), PipelineError> {
    let builder = ExpressionBuilder {};
    match sql {
        SelectItem::UnnamedExpr(sql_expr) => {
            match builder.parse_sql_expression(
                &BuilderExpressionType::FullExpression,
                sql_expr,
                schema,
            ) {
                Ok(expr) => Ok((sql_expr.to_string(), *expr.0)),
                Err(error) => Err(error),
            }
        }
        SelectItem::ExprWithAlias { expr, alias } => {
            match builder.parse_sql_expression(&BuilderExpressionType::FullExpression, expr, schema)
            {
                Ok(expr) => Ok((alias.value.clone(), *expr.0)),
                Err(error) => Err(error),
            }
        }
        SelectItem::Wildcard => Err(PipelineError::InvalidOperator("*".to_string())),
        SelectItem::QualifiedWildcard(ref object_name) => {
            Err(PipelineError::InvalidOperator(object_name.to_string()))
        }
    }
}
