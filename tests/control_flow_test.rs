use rustpy::config::Config;
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
        let mut interpreter = Interpreter::new(Config { trace: true });
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

#[test]
fn while_statement() {
    vec![
        (
            r###"
test = 1
while test < 100:
  test = test + 1
test
"###,
            100,
        ),
        (
            r###"
test = 1
while test < 0:
  test = test + 1
test
"###,
            1,
        ),
        (
            r###"
test = 0
while test < 100:
  local1 = 10
  test = local1 + test
test
"###,
            100,
        ),
        (
            r###"
test = 1
while test < 10:
  if test >= 5:
    break
  test = test + 1
test
"###,
            5,
        ),
        (
            r###"
test = 1
while test < 10:
  if test == 5:
    test = 20
    continue
  test = test + 1
test
"###,
            20,
        ),
    ]
    .iter()
    .for_each(|(source, expected)| {
        let mut interpreter = Interpreter::new(Config { trace: true });
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
