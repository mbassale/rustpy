use rustpy::object::Value;
use rustpy::Interpreter;

#[test]
fn if_statement() {
    vec![
        (
            r###"
if 1 < 0:
  1
elif 2 < 1:
  2
elif 2 < 0:
  3
else:
  4
"###,
            4,
        ),
        (
            r###"
if 1 > 0:
  1
else:
  2
"###,
            1,
        ),
        (
            r###"
if 1 == 0:
  1
elif 1 == 1:
  2
else:
  3
"###,
            2,
        ),
        (
            r###"
if True == True:
  if False != True:
    1
  else:
    2
else:
  3
"###,
            1,
        ),
    ]
    .iter()
    .for_each(|(source, expected)| {
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(source);
        assert!(result.is_ok());
        let value = result.unwrap();
        let value = match value {
            Value::Integer(value) => value,
            _ => panic!("Value not an integer"),
        };
        assert_eq!(value, *expected);
    });
}
