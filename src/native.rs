use crate::{errors::RuntimeError, interpreter::Value};
use dynfmt::{Format, PythonFormat};
use std::{thread, time};

pub fn sleep(args: &[Value]) -> Result<Value, RuntimeError> {
    match args {
        [Value::Integer(ms)] => {
            thread::sleep(time::Duration::from_millis(*ms as u64));
            Ok(Value::Nil)
        }
        _ => Err(RuntimeError("sleep expects duration in ms".to_string())),
    }
}

pub fn print(args: &[Value]) -> Result<Value, RuntimeError> {
    match args {
        [Value::String(message), rest @ ..] => {
            let vars: Vec<String> = rest.iter().map(|v| v.to_string()).collect();
            let output = PythonFormat
                .format(message, &vars)
                .map_err(|e| RuntimeError(format!("print format error: {e}")))?;
            println!("{output}");
            Ok(Value::Nil)
        }
        _ => Err(RuntimeError(
            "print(\"hello, {}\", var) is the format for print".to_string(),
        )),
    }
}
