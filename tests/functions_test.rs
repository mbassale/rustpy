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
