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
    pub fn evaluate(&self, input: String) -> String {
        match self {
            StringTransform::UpperCase => input.to_uppercase(),
            StringTransform::LowerCase => input.to_lowercase(),
        }
    }
}

pub enum StringExpr {
    Field(String),
    Literal(String),
    Transform(StringTransform, Box<StringExpr>),
}

impl StringExpr {
    pub fn evaluate(&self, row: &HashMap<String, String>) -> Option<String> {
        match self {
            StringExpr::Field(field) => row.get(field).map(String::to_string),
            StringExpr::Literal(ref literal) => Some(literal.to_string()),
            StringExpr::Transform(transform, ref expr) => Some(
                transform.evaluate(expr.evaluate(row)?.to_string()),
            ),
        }
    }
}

pub enum BoolExpr {
    Eq(StringExpr, StringExpr),
    Neq(StringExpr, StringExpr),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
}

impl BoolExpr {
    pub fn evaluate(&self, row: &HashMap<String, String>) -> bool {
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
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>();

        if query.evaluate(&row) {
            dbg!(row);
        }
    }

    Ok(())
}
