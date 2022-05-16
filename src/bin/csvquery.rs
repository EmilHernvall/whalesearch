use std::collections::HashMap;
use std::error::Error;

use serde_json::Value;

use lalrpop_util::lalrpop_mod;

lalrpop_util::lalrpop_mod!(pub dsl); // synthesized by LALRPOP

pub enum StringTransform {
    UpperCase,
    LowerCase,
}

impl StringTransform {
    pub fn evaluate(&self, input: Value) -> Option<Value> {
        let s = input.as_str()?;
        match self {
            StringTransform::UpperCase => Some(Value::String(s.to_uppercase())),
            StringTransform::LowerCase => Some(Value::String(s.to_lowercase())),
        }
    }
}

pub enum ValueExpr {
    Field(String),
    Value(Value),
    Transform(StringTransform, Box<ValueExpr>),
}

impl ValueExpr {
    pub fn evaluate(&self, row: &HashMap<String, Value>) -> Option<Value> {
        match self {
            ValueExpr::Field(field) => row.get(field).cloned(),
            ValueExpr::Value(value) => Some(value.clone()),
            ValueExpr::Transform(transform, ref expr) => {
                transform.evaluate(expr.evaluate(row)?)
            },
        }
    }
}

pub enum BoolExpr {
    Eq(ValueExpr, ValueExpr),
    Neq(ValueExpr, ValueExpr),
    Lt(ValueExpr, ValueExpr),
    Gt(ValueExpr, ValueExpr),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
}

impl BoolExpr {
    pub fn evaluate(&self, row: &HashMap<String, Value>) -> Option<bool> {
        match self {
            BoolExpr::Eq(lh, rh) => Some(rh.evaluate(row)? == lh.evaluate(row)?),
            BoolExpr::Neq(lh, rh) => Some(rh.evaluate(row)? != lh.evaluate(row)?),
            BoolExpr::Not(expr) => Some(!expr.evaluate(row)?),
            BoolExpr::And(lh, rh) => Some(lh.evaluate(row)? && rh.evaluate(row)?),
            BoolExpr::Or(lh, rh) => Some(lh.evaluate(row)? || rh.evaluate(row)?),
            BoolExpr::Lt(lh, rh) => {
                let fst = lh.evaluate(row)?.as_i64()?;
                let snd = rh.evaluate(row)?.as_i64()?;
                Some(fst < snd)
            },
            BoolExpr::Gt(lh, rh) => {
                let fst = lh.evaluate(row)?.as_i64()?;
                let snd = rh.evaluate(row)?.as_i64()?;
                Some(fst > snd)
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let query = std::env::args().nth(1).unwrap();
    let parser = dsl::BoolExprParser::new();
    let query: BoolExpr = parser.parse(&query).unwrap();

    let data = std::fs::read_to_string("whales.json")?;
    let whales: Vec<HashMap<String, Value>> = serde_json::from_str(&data)?;

    for whale in whales {
        if matches!(query.evaluate(&whale), Some(true)) {
            println!("{}", serde_json::to_string(&whale)?);
        }
    }

    Ok(())
}
