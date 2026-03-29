use std::sync::Arc;

use serial_test::serial;

use super::{EvalStateBuilder, Value};
use crate::Store;

#[test]
#[serial]
fn test_eval_state_builder() {
    let store = Arc::new(Store::open(None).expect("Failed to open store"));
    let _state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");
    // State should be dropped automatically
}

#[test]
#[serial]
fn test_simple_evaluation() {
    let store = Arc::new(Store::open(None).expect("Failed to open store"));
    let state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");

    let result = state
        .eval_from_string("1 + 2", "<eval>")
        .expect("Failed to evaluate expression");

    assert!(matches!(result, Value::Int(_)));
    if let Value::Int(value) = result {
        assert_eq!(value.as_int(), 3);
    } else {
        unreachable!();
    }
}

#[test]
#[serial]
fn test_value_types() {
    let store = Arc::new(Store::open(None).expect("Failed to open store"));
    let state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");

    // Test integer
    let int_val = state
        .eval_from_string("42", "<eval>")
        .expect("Failed to evaluate int");
    assert!(matches!(int_val, Value::Int(_)));
    if let Value::Int(value) = int_val {
        assert_eq!(value.as_int(), 42);
    } else {
        unreachable!();
    }

    // Test boolean
    let bool_val = state
        .eval_from_string("true", "<eval>")
        .expect("Failed to evaluate bool");
    assert!(matches!(bool_val, Value::Bool(_)));
    if let Value::Bool(value) = bool_val {
        assert_eq!(value.as_bool(), true);
    } else {
        unreachable!();
    }

    // Test string
    let string_val = state
        .eval_from_string("\"hello\"", "<eval>")
        .expect("Failed to evaluate string");
    assert!(matches!(string_val, Value::String(_)));
    if let Value::String(value) = string_val {
        assert_eq!(value.as_string(), "hello");
    } else {
        unreachable!();
    }
}

#[test]
#[serial]
fn test_value_formatting() {
    let store = Arc::new(Store::open(None).expect("Failed to open store"));
    let state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");

    // Test integer formatting
    let int_val = state
        .eval_from_string("42", "<eval>")
        .expect("Failed to evaluate int");
    assert_eq!(format!("{int_val}"), "42");
    assert_eq!(format!("{int_val:?}"), "Value::Int(NixInt(42))");

    // Test boolean formatting
    let true_val = state
        .eval_from_string("true", "<eval>")
        .expect("Failed to evaluate bool");
    assert_eq!(format!("{true_val}"), "true");
    assert_eq!(format!("{true_val:?}"), "Value::Bool(NixBool(true))");

    let false_val = state
        .eval_from_string("false", "<eval>")
        .expect("Failed to evaluate bool");
    assert_eq!(format!("{false_val}"), "false");
    assert_eq!(format!("{false_val:?}"), "Value::Bool(NixBool(false))");

    // Test string formatting
    let str_val = state
        .eval_from_string("\"hello world\"", "<eval>")
        .expect("Failed to evaluate string");
    assert_eq!(format!("{str_val}"), "hello world");
    assert_eq!(
        format!("{str_val:?}"),
        "Value::String(NixString(\"hello world\"))"
    );

    // Test string with quotes
    let quoted_str = state
        .eval_from_string("\"say \\\"hello\\\"\"", "<eval>")
        .expect("Failed to evaluate quoted string");
    assert_eq!(format!("{quoted_str}"), "say \"hello\"");
    assert_eq!(
        format!("{quoted_str:?}"),
        "Value::String(NixString(say \"hello\"))"
    );

    // Test null formatting
    let null_val = state
        .eval_from_string("null", "<eval>")
        .expect("Failed to evaluate null");
    assert_eq!(format!("{null_val}"), "null");
    assert_eq!(format!("{null_val:?}"), "Value::Null(NixNull)");

    // Test collection formatting
    let attrs_val = state
        .eval_from_string("{ a = 1; }", "<eval>")
        .expect("Failed to evaluate attrs");
    assert_eq!(format!("{attrs_val}"), "{ <attrs> }");
    assert_eq!(format!("{attrs_val:?}"), "Value::Attrs({ <attrs> })");

    let list_val = state
        .eval_from_string("[ 1 2 3 ]", "<eval>")
        .expect("Failed to evaluate list");
    assert_eq!(format!("{list_val}"), "[ <list> ]");
    assert_eq!(format!("{list_val:?}"), "Value::List([ <list> ])");
}
