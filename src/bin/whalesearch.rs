use std::error::Error;
use std::collections::HashMap;

use serde_json::Value;

use whalesearch;

fn main() -> Result<(), Box<dyn Error>> {
    let query = std::env::args().nth(1).unwrap();
    let parser = whalesearch::dsl::BoolExprParser::new();
    let query: whalesearch::BoolExpr = parser.parse(&query).unwrap();

    let code = dbg!(query.to_opcode());

    let data = std::fs::read_to_string("whales.json")?;
    let whales: Vec<HashMap<String, Value>> = serde_json::from_str(&data)?;

    for whale in whales {
        let result = whalesearch::evaluate(&code, &whale);
        if matches!(result, Some(Value::Bool(true))) {
            println!("{}", serde_json::to_string(&whale)?);
        }
    }

    Ok(())
}

