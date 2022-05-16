use std::collections::HashMap;

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

    pub fn to_opcode(&self) -> Vec<OpCode> {
        let mut result = vec![];
        match self {
            ValueExpr::Field(field) => {
                result.push(OpCode::PushField(field.to_string()));
            },
            ValueExpr::Value(value) => {
                result.push(OpCode::PushValue(value.clone()));
            },
            ValueExpr::Transform(StringTransform::UpperCase, ref expr) => {
                result.extend(expr.to_opcode());
                result.push(OpCode::UpperCase);
            },
            ValueExpr::Transform(StringTransform::LowerCase, ref expr) => {
                result.extend(expr.to_opcode());
                result.push(OpCode::LowerCase);
            },
        }
        result
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

    pub fn to_opcode(&self) -> Vec<OpCode> {
        let mut result = vec![];
        match self {
            BoolExpr::Eq(lh, rh) => {
                result.extend(lh.to_opcode());
                result.extend(rh.to_opcode());
                result.push(OpCode::Eq);
            },
            BoolExpr::Neq(lh, rh) => {
                result.extend(lh.to_opcode());
                result.extend(rh.to_opcode());
                result.push(OpCode::Neq);
            },
            BoolExpr::Not(expr) => {
                result.extend(expr.to_opcode());
                result.push(OpCode::Not);
            },
            BoolExpr::And(lh, rh) => {
                result.extend(lh.to_opcode());
                result.extend(rh.to_opcode());
                result.push(OpCode::And);
            },
            BoolExpr::Or(lh, rh) => {
                result.extend(lh.to_opcode());
                result.extend(rh.to_opcode());
                result.push(OpCode::Or);
            },
            BoolExpr::Lt(lh, rh) => {
                result.extend(lh.to_opcode());
                result.extend(rh.to_opcode());
                result.push(OpCode::Lt);
            },
            BoolExpr::Gt(lh, rh) => {
                result.extend(lh.to_opcode());
                result.extend(rh.to_opcode());
                result.push(OpCode::Gt);
            },
        }

        result
    }
}

#[derive(Debug)]
pub enum OpCode {
    Eq,
    Neq,
    Lt,
    Gt,
    Not,
    And,
    Or,
    PushField(String),
    PushValue(Value),
    UpperCase,
    LowerCase,
}

pub fn evaluate(code: &[OpCode], whale: &HashMap<String, Value>) -> Option<Value> {
    let mut stack = Vec::with_capacity(20);
    for op in code {
        match op {
            OpCode::Eq => {
                let fst = stack.pop()?;
                let snd = stack.pop()?;
                stack.push(Value::Bool(fst == snd));
            },
            OpCode::Neq => {
                let fst = stack.pop()?;
                let snd = stack.pop()?;
                stack.push(Value::Bool(fst == snd));
            },
            OpCode::Lt => {
                let fst = stack.pop()?.as_i64()?;
                let snd = stack.pop()?.as_i64()?;
                stack.push(Value::Bool(fst > snd));
            },
            OpCode::Gt => {
                let fst = stack.pop()?.as_i64()?;
                let snd = stack.pop()?.as_i64()?;
                stack.push(Value::Bool(fst < snd));
            },
            OpCode::Not => {
                let operand = stack.pop()?.as_bool()?;
                stack.push(Value::Bool(!operand));
            },
            OpCode::And => {
                let fst = stack.pop()?.as_bool()?;
                let snd = stack.pop()?.as_bool()?;
                stack.push(Value::Bool(snd && fst));
            },
            OpCode::Or => {
                let fst = stack.pop()?.as_bool()?;
                let snd = stack.pop()?.as_bool()?;
                stack.push(Value::Bool(snd || fst));
            },
            OpCode::PushField(ref field) => {
                let value = whale.get(field)?;
                stack.push(value.clone());
            },
            OpCode::PushValue(ref value) => {
                stack.push(value.clone());
            },
            OpCode::UpperCase => {
                let operand = stack.pop()?.as_str().map(|x| x.to_uppercase())?;
                stack.push(Value::String(operand));
            },
            OpCode::LowerCase => {
                let operand = stack.pop()?.as_str().map(|x| x.to_lowercase())?;
                stack.push(Value::String(operand));
            },
        }
    }

    stack.pop()
}
