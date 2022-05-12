use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use serde::{Serialize, Deserialize};

use lalrpop_util::lalrpop_mod;

lalrpop_util::lalrpop_mod!(pub dsl); // synthesized by LALRPOP

pub enum StringTransform {
    UpperCase,
    LowerCase,
}

impl StringTransform {
    pub fn evaluate(&self, input: Value) -> Value {
        match self {
            StringTransform::UpperCase => input.map_string(|x| x.to_uppercase()),
            StringTransform::LowerCase => input.map_string(|x| x.to_lowercase()),
        }
    }
}

#[derive(Debug,Clone)]
pub enum Value {
    Literal(String),
    Number(f64),
}

impl PartialEq for Value {
    fn eq(&self, rhs: &Value) -> bool {
        match (self, rhs) {
            (Value::Literal(a), Value::Literal(b)) if a == b => true,
            (Value::Number(a), Value::Number(b)) if a == b => true,
            (Value::Literal(a), Value::Number(b)) |
            (Value::Number(b), Value::Literal(a)) => {
                let a: f64 = match a.parse() {
                    Ok(x) => x,
                    Err(_e) => return false,
                };
                a == *b
            },
            _ => false,
        }
    }
}

impl Value {
    fn map_string(self, f: impl Fn(String) -> String) -> Value {
        match self {
            Value::Literal(s) => Value::Literal(f(s)),
            _ => self,
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
            ValueExpr::Field(field) => row.get(field).map(|x| x.clone()),
            ValueExpr::Value(value) => Some(value.clone()),
            ValueExpr::Transform(transform, ref expr) => Some(
                transform.evaluate(expr.evaluate(row)?),
            ),
        }
    }
}

pub enum BoolExpr {
    Eq(ValueExpr, ValueExpr),
    Neq(ValueExpr, ValueExpr),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
}

impl BoolExpr {
    pub fn evaluate(&self, row: &HashMap<String, Value>) -> bool {
        match self {
            BoolExpr::Eq(lh, rh) => rh.evaluate(row) == lh.evaluate(row),
            BoolExpr::Neq(lh, rh) => rh.evaluate(row) != lh.evaluate(row),
            BoolExpr::Not(expr) => !expr.evaluate(row),
            BoolExpr::And(lh, rh) => lh.evaluate(row) && rh.evaluate(row),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = std::env::args().nth(1).unwrap();

    let query = std::env::args().nth(2).unwrap();
    let parser = dsl::BoolExprParser::new();
    let query: BoolExpr = parser.parse(&query).unwrap();

    let file = File::open(filename)?;
    let file = BufReader::new(file);

    let mut lines = file.lines().filter_map(|x| x.ok());
    let header = lines.next().unwrap();
    let header = header.split(",").collect::<Vec<_>>();

    for line in lines {
        let fields = line.split(",").collect::<Vec<_>>();
        if fields.len() != header.len() {
            continue;
        }

        let row = header.iter()
            .zip(fields.iter())
            .map(|(k, v)| (k.to_string(), Value::Literal(v.to_string())))
            .collect::<HashMap<_, _>>();

        if query.evaluate(&row) {
            dbg!(row);
        }
    }

    Ok(())
}
