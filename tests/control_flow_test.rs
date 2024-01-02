use rustpy::Interpreter;

#[test]
fn if_statement() {
    vec![
        r###"
if 1 < 0:
  1
elif 2 < 1:
  2
else:
  3
"###,
    ]
    .iter()
    .for_each(|source| {
        let mut interpreter = Interpreter::new();
        let result = interpreter.run(source);
        assert!(result.is_ok());
    });
}
