use crate::common::error::{DozerSqlError, Result};
use crate::pipeline::expression::comparison::{Eq, Gt, Gte, Lt, Lte, Ne};
use crate::pipeline::expression::logical::{And, Or};
use crate::pipeline::expression::mathematical::{Add, Div, Mod, Mul, Sub};
use crate::pipeline::expression::operator::{Column, Expression};
use crate::pipeline::processor::selection::SelectionProcessor;
use dozer_core::dag::dag::Dag;
use dozer_core::dag::dag::NodeType;
use dozer_core::dag::node::Processor;
use sqlparser::ast::{
    BinaryOperator, Expr as SqlExpr, Query, Select, SelectItem, SetExpr, Statement,
    Value as SqlValue,
};

pub struct PipelineBuilder {}

impl PipelineBuilder {
    pub fn statement_to_pipeline(statement: Statement) -> Result<Dag> {
        match statement {
            Statement::Query(query) => PipelineBuilder::query_to_pipeline(*query),
            _ => Err(DozerSqlError::NotImplemented(
                "Unsupported Query.".to_string(),
            )),
        }
    }

    pub fn query_to_pipeline(query: Query) -> Result<Dag> {
        PipelineBuilder::set_expr_to_pipeline(*query.body)
    }

    fn set_expr_to_pipeline(set_expr: SetExpr) -> Result<Dag> {
        match set_expr {
            SetExpr::Select(s) => PipelineBuilder::select_to_pipeline(*s),
            SetExpr::Query(q) => PipelineBuilder::query_to_pipeline(*q),
            _ => Err(DozerSqlError::NotImplemented(
                "Unsupported Query.".to_string(),
            )),
        }
    }

    fn select_to_pipeline(select: Select) -> Result<Dag> {
        // Where clause
        let selection_processor = PipelineBuilder::selection_to_processor(select.selection)?;

        // Select clause
        // let projection_processor = PipelineBuilder::projection_to_processor(select.projection)?;

        Ok(Dag::new())
    }

    fn selection_to_processor(selection: Option<SqlExpr>) -> Result<Box<dyn Processor>> {
        match selection {
            Some(expression) => {
                let operator = PipelineBuilder::parse_sql_expression(expression)?;
                Ok(Box::new(SelectionProcessor::new(0, None, None)))
            }
            _ => Err(DozerSqlError::NotImplemented(
                "Unsupported WHERE clause.".to_string(),
            )),
        }
    }

    fn projection_to_processor(projection: Vec<SelectItem>) -> Result<Box<dyn Processor>> {
        Err(DozerSqlError::NotImplemented(
            "Unsupported SELECT.".to_string(),
        ))
    }

    fn parse_sql_expression(expression: SqlExpr) -> Result<Box<dyn Expression>> {
        match expression {
            SqlExpr::Identifier(i) => Ok(Box::new(Column::new(i.to_string()))),
            SqlExpr::Value(SqlValue::Number(n, _)) => Ok(PipelineBuilder::parse_sql_number(&n)?),
            SqlExpr::Value(SqlValue::SingleQuotedString(s) | SqlValue::DoubleQuotedString(s)) => {
                Ok(Box::new(s))
            }
            SqlExpr::BinaryOp { left, op, right } => {
                Ok(PipelineBuilder::parse_sql_binary_op(*left, op, *right)?)
            }

            _ => Err(DozerSqlError::NotImplemented(
                "Unsupported expression.".to_string(),
            )),
        }
    }

    fn parse_sql_number(n: &str) -> Result<Box<dyn Expression>> {
        match n.parse::<i64>() {
            Ok(n) => Ok(Box::new(n)),
            Err(_) => match n.parse::<f64>() {
                Ok(f) => Ok(Box::new(f)),
                Err(_) => Err(DozerSqlError::NotImplemented(
                    "Value is not numeric.".to_string(),
                )),
            },
        }
    }

    fn parse_sql_binary_op(
        left: SqlExpr,
        op: BinaryOperator,
        right: SqlExpr,
    ) -> Result<Box<dyn Expression>> {
        let left_op = PipelineBuilder::parse_sql_expression(left)?;
        let right_op = PipelineBuilder::parse_sql_expression(right)?;
        match op {
            BinaryOperator::Gt => Ok(Box::new(Gt::new(left_op, right_op))),
            BinaryOperator::GtEq => Ok(Box::new(Gte::new(left_op, right_op))),
            BinaryOperator::Lt => Ok(Box::new(Lt::new(left_op, right_op))),
            BinaryOperator::LtEq => Ok(Box::new(Lte::new(left_op, right_op))),
            BinaryOperator::Eq => Ok(Box::new(Eq::new(left_op, right_op))),
            BinaryOperator::NotEq => Ok(Box::new(Ne::new(left_op, right_op))),
            BinaryOperator::Plus => Ok(Box::new(Add::new(left_op, right_op))),
            BinaryOperator::Minus => Ok(Box::new(Sub::new(left_op, right_op))),
            BinaryOperator::Multiply => Ok(Box::new(Mul::new(left_op, right_op))),
            BinaryOperator::Divide => Ok(Box::new(Div::new(left_op, right_op))),
            BinaryOperator::Modulo => Ok(Box::new(Mod::new(left_op, right_op))),
            BinaryOperator::And => Ok(Box::new(And::new(left_op, right_op))),
            BinaryOperator::Or => Ok(Box::new(Or::new(left_op, right_op))),
            BinaryOperator::BitwiseAnd => Err(DozerSqlError::NotImplemented(
                "Unsupported operator BITWISE AND.".to_string(),
            )),
            BinaryOperator::BitwiseOr => Err(DozerSqlError::NotImplemented(
                "Unsupported operator BITWISE OR.".to_string(),
            )),
            BinaryOperator::StringConcat => Err(DozerSqlError::NotImplemented(
                "Unsupported operator CONCAT.".to_string(),
            )),
            // BinaryOperator::PGRegexMatch => Err(DozerSqlError::NotImplemented("Unsupported operator.".to_string())),
            // BinaryOperator::PGRegexIMatch => Err(DozerSqlError::NotImplemented("Unsupported operator.".to_string())),
            // BinaryOperator::PGRegexNotMatch => Err(DozerSqlError::NotImplemented("Unsupported operator.".to_string())),
            // BinaryOperator::PGRegexNotIMatch => Err(DozerSqlError::NotImplemented("Unsupported operator.".to_string())),
            // BinaryOperator::PGBitwiseShiftRight => Err(DozerSqlError::NotImplemented("Unsupported operator.".to_string())),
            // BinaryOperator::PGBitwiseShiftLeft => Err(DozerSqlError::NotImplemented("Unsupported operator.".to_string())),
            _ => Err(DozerSqlError::NotImplemented(
                "Unsupported operator.".to_string(),
            )),
        }
    }
}

use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

#[test]
fn test_pipeline_builder() {
    let sql = "SELECT Country, COUNT(CustomerID), SUM(Spending) \
                            FROM Customers \
                            WHERE Spending >= 1000 \
                            GROUP BY Country \
                            HAVING COUNT(CustomerID) > 1;";

    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

    let ast = Parser::parse_sql(&dialect, sql).unwrap();
    println!("AST: {:?}", ast);

    let statement = &ast[0];
    let pipeline = PipelineBuilder::statement_to_pipeline(statement.clone()).unwrap();
}