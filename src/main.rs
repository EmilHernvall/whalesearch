use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use serde::{Serialize, Deserialize};

use lalrpop_util::lalrpop_mod;

lalrpop_util::lalrpop_mod!(pub dsl); // synthesized by LALRPOP

#[derive(Serialize,Deserialize)]
pub enum StringExpr {
    Field(String),
    Literal(String),
}

impl StringExpr {
    pub fn evaluate<'a>(&'a self, row: &'a HashMap<String, String>) -> Option<&'a String> {
        match self {
            StringExpr::Field(field) => row.get(field),
            StringExpr::Literal(ref literal) => Some(literal),
        }
    }
}

#[derive(Serialize,Deserialize)]
pub enum BoolExpr {
    Eq(StringExpr, StringExpr),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
}

impl BoolExpr {
    pub fn evaluate(&self, row: &HashMap<String, String>) -> bool {
        match self {
            BoolExpr::Eq(lh, rh) => rh.evaluate(row) == lh.evaluate(row),
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
