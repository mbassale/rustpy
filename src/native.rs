use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::object::Object;
use crate::object::Value;

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub function: Arc<dyn Fn(Vec<Object>) -> Object + Send + Sync>,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd for NativeFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<native:{}:{}>", self.name, self.arity))
    }
}

fn abs(args: Vec<Object>) -> Object {
    let obj = &args[0];
    match obj.value {
        Value::Integer(value) => {
            let value = if value < 0 { -value } else { value };
            Object::new(Value::Integer(value))
        }
        Value::Float(value) => {
            let value = if value < 0.0 { -value } else { value };
            Object::new(Value::Float(value))
        }
        _ => Object::new(Value::Integer(0)),
    }
}

fn print(args: Vec<Object>) -> Object {
    let message = args
        .iter()
        .map(|obj| obj.value.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", message);
    Object::new_none()
}

pub fn init_native_function_registry() -> HashMap<String, NativeFunction> {
    let mut native_functions = HashMap::new();

    native_functions.insert(
        String::from("print"),
        NativeFunction {
            name: String::from("print"),
            arity: usize::MAX,
            function: Arc::new(print),
        },
    );
    native_functions.insert(
        String::from("abs"),
        NativeFunction {
            name: String::from("abs"),
            arity: 1,
            function: Arc::new(abs),
        },
    );

    native_functions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_registry() {
        let nfr = init_native_function_registry();
        let result = nfr.get("print");
        assert!(result.is_some());
        let print_func_obj = result.unwrap();
        assert_eq!(
            (print_func_obj.function.as_ref())(vec![Object::new_true()]),
            Object::new_none()
        );
    }

    #[test]
    fn test_abs() {
        let nfr = init_native_function_registry();
        let result = nfr.get("abs");
        assert!(result.is_some());
        let abs_func_obj = result.unwrap();
        let abs_func = abs_func_obj.function.as_ref();
        vec![(0, 0), (-1, 1), (1, 1)]
            .into_iter()
            .for_each(|(value, expected)| {
                let args = vec![Object::new(Value::Integer(value))];
                let actual = abs_func(args);
                if let Value::Integer(actual) = actual.value {
                    assert_eq!(actual, expected);
                } else {
                    panic!("Result not an integer");
                }
            });
        vec![(0.0, 0.0), (-1.0, 1.0), (1.0, 1.0)]
            .into_iter()
            .for_each(|(value, expected)| {
                let args = vec![Object::new(Value::Float(value))];
                let actual = abs_func(args);
                if let Value::Float(actual) = actual.value {
                    assert_eq!(actual, expected);
                } else {
                    panic!("Result not a float");
                }
            });
    }
}
