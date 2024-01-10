use rustpy::object::Value;
use rustpy::Interpreter;

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
        let mut interpreter = Interpreter::new();
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
    ]
    .into_iter()
    .for_each(|(source, expected)| {
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(source);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, expected);
    });
}
