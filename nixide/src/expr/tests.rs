use std::sync::Arc;

use serial_test::serial;

use super::{EvalStateBuilder, ValueType};
use crate::{ErrorContext, Store};

#[test]
#[serial]
fn test_context_creation() {
    let _ctx = ErrorContext::new().expect("Failed to create context");
    // Context should be dropped automatically
}

#[test]
#[serial]
fn test_eval_state_builder() {
    let ctx = Arc::new(ErrorContext::new().expect("Failed to create context"));
    let store = Arc::new(Store::open(&ctx, None).expect("Failed to open store"));
    let _state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");
    // State should be dropped automatically
}

#[test]
#[serial]
fn test_simple_evaluation() {
    let ctx = Arc::new(ErrorContext::new().expect("Failed to create context"));
    let store = Arc::new(Store::open(&ctx, None).expect("Failed to open store"));
    let state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");

    let result = state
        .eval_from_string("1 + 2", "<eval>")
        .expect("Failed to evaluate expression");

    assert_eq!(result.value_type(), ValueType::Int);
    assert_eq!(result.as_int().expect("Failed to get int value"), 3);
}

#[test]
#[serial]
fn test_value_types() {
    let ctx = Arc::new(ErrorContext::new().expect("Failed to create context"));
    let store = Arc::new(Store::open(&ctx, None).expect("Failed to open store"));
    let state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");

    // Test integer
    let int_val = state
        .eval_from_string("42", "<eval>")
        .expect("Failed to evaluate int");
    assert_eq!(int_val.value_type(), ValueType::Int);
    assert_eq!(int_val.as_int().expect("Failed to get int"), 42);

    // Test boolean
    let bool_val = state
        .eval_from_string("true", "<eval>")
        .expect("Failed to evaluate bool");
    assert_eq!(bool_val.value_type(), ValueType::Bool);
    assert!(bool_val.as_bool().expect("Failed to get bool"));

    // Test string
    let str_val = state
        .eval_from_string("\"hello\"", "<eval>")
        .expect("Failed to evaluate string");
    assert_eq!(str_val.value_type(), ValueType::String);
    assert_eq!(str_val.as_string().expect("Failed to get string"), "hello");
}

#[test]
#[serial]
fn test_value_formatting() {
    let ctx = Arc::new(ErrorContext::new().expect("Failed to create context"));
    let store = Arc::new(Store::open(&ctx, None).expect("Failed to open store"));
    let state = EvalStateBuilder::new(&store)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build state");

    // Test integer formatting
    let int_val = state
        .eval_from_string("42", "<eval>")
        .expect("Failed to evaluate int");
    assert_eq!(format!("{int_val}"), "42");
    assert_eq!(format!("{int_val:?}"), "Value::Int(42)");
    assert_eq!(int_val.to_nix_string().expect("Failed to format"), "42");

    // Test boolean formatting
    let bool_val = state
        .eval_from_string("true", "<eval>")
        .expect("Failed to evaluate bool");
    assert_eq!(format!("{bool_val}"), "true");
    assert_eq!(format!("{bool_val:?}"), "Value::Bool(true)");
    assert_eq!(bool_val.to_nix_string().expect("Failed to format"), "true");

    let false_val = state
        .eval_from_string("false", "<eval>")
        .expect("Failed to evaluate bool");
    assert_eq!(format!("{false_val}"), "false");
    assert_eq!(
        false_val.to_nix_string().expect("Failed to format"),
        "false"
    );

    // Test string formatting
    let str_val = state
        .eval_from_string("\"hello world\"", "<eval>")
        .expect("Failed to evaluate string");
    assert_eq!(format!("{str_val}"), "hello world");
    assert_eq!(format!("{str_val:?}"), "Value::String(\"hello world\")");
    assert_eq!(
        str_val.to_nix_string().expect("Failed to format"),
        "\"hello world\""
    );

    // Test string with quotes
    let quoted_str = state
        .eval_from_string("\"say \\\"hello\\\"\"", "<eval>")
        .expect("Failed to evaluate quoted string");
    assert_eq!(format!("{quoted_str}"), "say \"hello\"");
    assert_eq!(
        quoted_str.to_nix_string().expect("Failed to format"),
        "\"say \\\"hello\\\"\""
    );

    // Test null formatting
    let null_val = state
        .eval_from_string("null", "<eval>")
        .expect("Failed to evaluate null");
    assert_eq!(format!("{null_val}"), "null");
    assert_eq!(format!("{null_val:?}"), "Value::Null");
    assert_eq!(null_val.to_nix_string().expect("Failed to format"), "null");

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
