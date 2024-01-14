use rustpy::config::Config;
use rustpy::object::Value;
use rustpy::Interpreter;

mod common;

#[test]
fn function_definition() {
    vec![(
        r###"
def test(arg1, arg2):
  return arg1 + arg2

True
"###,
        Value::True,
    )]
    .into_iter()
    .for_each(|(source, expected)| {
        let mut interpreter = Interpreter::new(Config { trace: true });
        let result = interpreter.run(source);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, expected);
    });
}

#[test]
fn function_call() {
    vec![
        (
            r###"
def test():
  True

 test()
 "###,
            Value::True,
        ),
        (
            r###"
 def test():
   1 + 1

 test()
 "###,
            Value::Integer(2),
        ),
        (
            r###"
def two():
  return 1 + 1

def six():
  return 6

result = 1
result = two()
result = result * six()
result
"###,
            Value::Integer(12),
        ),
        (
            r###"
def double(n):
  return 2 * n

double(10)
        "###,
            Value::Integer(20),
        ),
        (
            &common::load_source("tests/test0_functions.py"),
            Value::Integer(20),
        ),
    ]
    .into_iter()
    .for_each(|(source, expected)| {
        let mut interpreter = Interpreter::new(Config { trace: true });
        let result = interpreter.run(source);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, expected);
    });
}
