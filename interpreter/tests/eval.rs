use interpreter::{
    eval,
    eval::object::{Object, Objecter, environment::Environment},
    lexer::Lexer,
    parser::Parser,
};

fn run(input: &str) -> String {
    let env = Environment::new();
    let lexer = Lexer::new(input);

    let mut parser = match Parser::new(lexer) {
        Ok(parser) => parser,
        Err(err) => return format!("lexer/parser error: {err}"),
    };

    match parser.parse_program() {
        Ok(program) => {
            let result = eval::evaluate(&program, &env);
            let mut out = env.borrow().take_output();

            match result {
                Ok(Object::Null(_)) => {}
                Ok(obj) => out.push_str(&obj.inspect_value()),
                Err(err) => out.push_str(&err.inspect_value()),
            }

            out
        }
        Err(errors) => {
            let errs = errors
                .iter()
                .map(|err| format!("\t{err}"))
                .collect::<Vec<_>>()
                .join("\n");
            format!("parser has {} error(s):\n{}", errors.len(), errs)
        }
    }
}

struct Case {
    name: &'static str,
    input: &'static str,
    output: &'static str,
}

const CASES: &[Case] = &[
    // integers
    Case { name: "int_literal_positive", input: "5", output: "5" },
    Case { name: "int_literal_positive_10", input: "10", output: "10" },
    Case { name: "int_prefix_negative", input: "-5", output: "-5" },
    Case { name: "int_prefix_negative_10", input: "-10", output: "-10" },
    Case { name: "int_add_sub_chain", input: "5 + 5 + 5 + 5 - 10", output: "10" },
    Case { name: "int_mul_chain", input: "2 * 2 * 2 * 2 * 2", output: "32" },
    Case { name: "int_negatives_sum", input: "-50 + 100 + -50", output: "0" },
    Case { name: "int_mul_before_add", input: "5 * 2 + 10", output: "20" },
    Case { name: "int_add_before_mul", input: "5 + 2 * 10", output: "25" },
    Case { name: "int_mixed_precedence", input: "20 + 2 * -10", output: "0" },
    Case { name: "int_div_mul_add", input: "50 / 2 * 2 + 10", output: "60" },
    Case { name: "int_paren_mul", input: "2 * (5 + 10)", output: "30" },
    Case { name: "int_triple_mul_add", input: "3 * 3 * 3 + 10", output: "37" },
    Case { name: "int_paren_triple_mul_add", input: "3 * (3 * 3) + 10", output: "37" },
    Case { name: "int_complex_expr", input: "(5 + 10 * 2 + 15 / 3) * 2 + -10", output: "50" },

    // booleans
    Case { name: "bool_true", input: "true", output: "true" },
    Case { name: "bool_false", input: "false", output: "false" },
    Case { name: "bool_lt_true", input: "1 < 2", output: "true" },
    Case { name: "bool_gt_false", input: "1 > 2", output: "false" },
    Case { name: "bool_lt_eq_false", input: "1 < 1", output: "false" },
    Case { name: "bool_gt_eq_false", input: "1 > 1", output: "false" },
    Case { name: "bool_eq_true", input: "1 == 1", output: "true" },
    Case { name: "bool_neq_false", input: "1 != 1", output: "false" },
    Case { name: "bool_eq_false", input: "1 == 2", output: "false" },
    Case { name: "bool_neq_true", input: "1 != 2", output: "true" },
    Case { name: "bool_true_eq_true", input: "true == true", output: "true" },
    Case { name: "bool_false_eq_false", input: "false == false", output: "true" },
    Case { name: "bool_true_eq_false", input: "true == false", output: "false" },
    Case { name: "bool_true_neq_false", input: "true != false", output: "true" },
    Case { name: "bool_false_neq_true", input: "false != true", output: "true" },
    Case { name: "bool_group_eq_true", input: "(1 < 2) == true", output: "true" },
    Case { name: "bool_group_eq_false", input: "(1 < 2) == false", output: "false" },
    Case { name: "bool_group_gt_eq_true", input: "(1 > 2) == true", output: "false" },
    Case { name: "bool_group_gt_eq_false", input: "(1 > 2) == false", output: "true" },

    // bang
    Case { name: "bang_true", input: "!true", output: "false" },
    Case { name: "bang_false", input: "!false", output: "true" },
    Case { name: "bang_int", input: "!5", output: "false" },
    Case { name: "bang_bang_true", input: "!!true", output: "true" },
    Case { name: "bang_bang_false", input: "!!false", output: "false" },
    Case { name: "bang_bang_int", input: "!!5", output: "true" },

    // chained_booleans
    Case { name: "and_both_true", input: "true && true", output: "true" },
    Case { name: "and_left_true_evaluates_right", input: "true && false", output: "false" },
    Case { name: "and_left_false_short_circuits", input: "false && foo", output: "false" },
    Case { name: "and_chain", input: "true && true && false", output: "false" },
    Case { name: "or_left_true_short_circuits", input: "true || foo", output: "true" },
    Case { name: "or_left_false_evaluates_right", input: "false || true", output: "true" },
    Case { name: "or_chain", input: "false || false || true", output: "true" },
    Case { name: "and_with_comparisons", input: "1 < 2 && 3 < 4", output: "true" },
    Case { name: "or_with_comparisons", input: "0 == 1 || 2 == 2", output: "true" },

    // if_else
    Case { name: "if_true_returns_consequence", input: "if (true) { 10 }", output: "10" },
    Case { name: "if_false_returns_null", input: "if (false) { 10 }", output: "" },
    Case { name: "if_truthy_int", input: "if (1) { 10 }", output: "10" },
    Case { name: "if_lt_true", input: "if (1 < 2) { 10 }", output: "10" },
    Case { name: "if_gt_false_null", input: "if (1 > 2) { 10 }", output: "" },
    Case { name: "if_else_falls_through", input: "if (1 > 2) { 10 } else { 20 }", output: "20" },
    Case { name: "if_else_true_branch", input: "if (1 < 2) { 10 } else { 20 }", output: "10" },

    // return
    Case { name: "return_simple", input: "return 10;", output: "10" },
    Case { name: "return_ignores_trailing", input: "return 10; 9;", output: "10" },
    Case { name: "return_expr", input: "return 2 * 5; 9;", output: "10" },
    Case { name: "return_after_stmt", input: "9; return 2 * 5; 9;", output: "10" },
    Case {
        name: "return_nested_if",
        input: "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
        output: "10",
    },

    // errors
    Case { name: "err_int_plus_bool", input: "5 + true;", output: "ERROR: type mismatch: Integer + Boolean" },
    Case { name: "err_int_plus_bool_trailing", input: "5 + true; 5;", output: "ERROR: type mismatch: Integer + Boolean" },
    Case { name: "err_negate_bool", input: "-true", output: "ERROR: unknown operator: -Boolean" },
    Case { name: "err_bool_plus_bool", input: "true + false;", output: "ERROR: unknown operator: Boolean + Boolean" },
    Case { name: "err_bool_plus_bool_leading", input: "5; true + false; 5", output: "ERROR: unknown operator: Boolean + Boolean" },
    Case { name: "err_bool_plus_bool_in_if", input: "if (10 > 1) { true + false; }", output: "ERROR: unknown operator: Boolean + Boolean" },
    Case {
        name: "err_bool_plus_bool_in_return",
        input: "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
        output: "ERROR: unknown operator: Boolean + Boolean",
    },
    Case { name: "err_identifier_not_found", input: "foobar", output: "ERROR: identifier not found: foobar" },
    Case { name: "err_assign_undeclared", input: "x = 5", output: "ERROR: identifier not found: x" },
    Case { name: "err_string_minus_string", input: "\"hello\" - \"world\"", output: "ERROR: unknown operator: String - String" },
    Case {
        name: "err_function_as_hash_key",
        input: r#"{"name": "Monkey"}[fn(x) { x }];"#,
        output: "ERROR: Function is unusable as a hash key",
    },
    Case {
        name: "err_push_wrong_arity",
        input: "push([1, 2], 3, 4)",
        output: "ERROR: wrong number of arguments to `push`. got=3, want=2",
    },
    Case {
        name: "err_push_wrong_type",
        input: "push(5, 1)",
        output: "ERROR: argument to `push` not supported, expected Array, got Integer",
    },
    Case {
        name: "err_index_assign_out_of_bounds",
        input: "let a = [1, 2, 3]; a[5] = 9",
        output: "ERROR: index 5 out of bounds for Array of length 3, use `push` to grow",
    },
    Case {
        name: "err_index_assign_non_integer",
        input: r#"let a = [1, 2, 3]; a["x"] = 9"#,
        output: "ERROR: array index must be an Integer, got String",
    },
    Case {
        name: "err_index_assign_unsupported_target",
        input: "let n = 5; n[0] = 1",
        output: "ERROR: index assignment not supported: Integer",
    },
    Case {
        name: "err_hash_assign_function_key",
        input: r#"let h = {}; h[fn(x) { x }] = 1"#,
        output: "ERROR: Function is unusable as a hash key",
    },

    // let
    Case { name: "let_simple", input: "let a = 5; a;", output: "5" },
    Case { name: "let_with_expr", input: "let a = 5 * 5; a;", output: "25" },
    Case { name: "let_from_another", input: "let a = 5; let b = a; b;", output: "5" },
    Case { name: "let_chained", input: "let a = 5; let b = a; let c = a + b + 5; c;", output: "15" },

    // reassignment
    Case { name: "reassign_literal", input: "let a = 5; a = 10; a", output: "10" },
    Case { name: "reassign_from_var", input: "let a = 1; let b = 2; a = b; a", output: "2" },
    Case { name: "reassign_self_increment", input: "let a = 0; a = a + 1; a = a + 1; a", output: "2" },
    Case {
        name: "reassign_in_for_loop",
        input: "let total = 0; for x in [1, 2, 3] { total = total + x; } total",
        output: "6",
    },
    Case {
        name: "reassign_captured_by_closure",
        input: "let a = 0; let f = fn() { a = 9; }; f(); a",
        output: "9",
    },

    // index_assignment
    Case { name: "index_assign_array_element", input: "let a = [1, 2, 3]; a[0] = 9; a[0]", output: "9" },
    Case { name: "index_assign_array_sum", input: "let a = [1, 2, 3]; a[2] = 30; a[0] + a[1] + a[2]", output: "33" },
    Case { name: "index_assign_array_via_var", input: "let a = [1, 2, 3]; let i = 1; a[i] = a[i] + 5; a[1]", output: "7" },
    Case { name: "index_assign_hash_existing_key", input: r#"let h = {"a": 1}; h["a"] = 5; h["a"]"#, output: "5" },
    Case { name: "index_assign_hash_new_key", input: r#"let h = {"a": 1}; h["b"] = 2; h["a"] + h["b"]"#, output: "3" },
    Case { name: "index_assign_hash_empty", input: "let h = {}; h[1] = 10; h[1]", output: "10" },
    Case { name: "index_assign_hash_dot", input: r#"let h = {"x": 1}; h.x = 42; h.x"#, output: "42" },
    Case {
        name: "index_assign_nested_array_of_hashes",
        input: r#"let people = [{"name": "a"}, {"name": "b"}]; people[1]["name"] = "z"; len(people[1]["name"])"#,
        output: "1",
    },
    Case {
        name: "index_assign_shares_underlying_array",
        input: "let a = [1, 2, 3]; let b = a; a[0] = 99; b[0]",
        output: "99",
    },

    // functions
    Case { name: "function_literal_inspect", input: "fn(x) { x + 2; }", output: "fn(x) {\n(x + 2)\n}" },
    Case { name: "function_identity", input: "let identity = fn(x) { x; }; identity(5);", output: "5" },
    Case { name: "function_identity_with_return", input: "let identity = fn(x) { return x; }; identity(5);", output: "5" },
    Case { name: "function_double", input: "let double = fn(x) { x * 2; }; double(5);", output: "10" },
    Case { name: "function_add", input: "let add = fn(x, y) { x + y; }; add(5, 5);", output: "10" },
    Case {
        name: "function_add_nested_call",
        input: "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
        output: "20",
    },
    Case { name: "function_immediately_invoked", input: "fn(x) { x; }(5)", output: "5" },

    // closures
    Case {
        name: "closure_new_adder",
        input: "let newAdder = fn(x) { fn(y) { x + y }; }; let addTwo = newAdder(2); addTwo(2);",
        output: "4",
    },

    // strings
    Case { name: "string_literal", input: "\"hello world\"", output: "hello world" },
    Case { name: "string_concat", input: "\"hello\" + \" \" + \"world\"", output: "hello world" },
    Case { name: "string_index_first_char", input: r#" let s = "xyz"; print(s[0]); "#, output: "x" },
    Case { name: "string_index_second_char", input: r#" let s = "xyz"; print(s[1]); "#, output: "y" },
    Case { name: "string_index_offset", input: r#" print("xyzyzyzywdq"[9]); "#, output: "d" },
    Case { name: "string_index_equality", input: r#" let s = "xyz"; print(s[0] == "x"); "#, output: "true" },
    Case { name: "string_looping", input: r#" for c in "xyz" { print(c) }; "#, output: "xyz" },

    // builtins
    Case { name: "len_empty_string", input: r#"len("")"#, output: "0" },
    Case { name: "len_string", input: r#"len("four")"#, output: "4" },
    Case { name: "len_longer_string", input: r#"len("hello world")"#, output: "11" },
    Case { name: "len_wrong_type", input: "len(1)", output: "ERROR: argument to `len` not supported, expected String or Array, got Integer" },
    Case { name: "len_wrong_arity", input: r#"len("one", "two")"#, output: "ERROR: wrong number of arguments to `len`. got=2, want=1" },
    Case { name: "min_basic", input: "min(1, 2)", output: "1" },
    Case { name: "max_basic", input: "max(1, 2)", output: "2" },
    Case { name: "min_reversed", input: "min(1, 103)", output: "1" },
    Case { name: "max_reversed", input: "max(103, 1)", output: "103" },

    // arrays
    Case { name: "array_literal_with_exprs", input: "[1, 2 * 2, 3 + 3]", output: "[1, 4, 6]" },
    Case { name: "array_index_zero", input: "[1, 2, 3][0]", output: "1" },
    Case { name: "array_index_one", input: "[1, 2, 3][1]", output: "2" },
    Case { name: "array_index_two", input: "[1, 2, 3][2]", output: "3" },
    Case { name: "array_index_via_var", input: "let i = 0; [1][i];", output: "1" },
    Case { name: "array_index_expr", input: "[1, 2, 3][1 + 1];", output: "3" },
    Case { name: "array_index_named_var", input: "let myArray = [1, 2, 3]; myArray[2];", output: "3" },
    Case {
        name: "array_index_sum",
        input: "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
        output: "6",
    },
    Case {
        name: "array_index_by_element",
        input: "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
        output: "2",
    },
    Case { name: "array_index_out_of_bounds", input: "[1, 2, 3][3]", output: "" },
    Case { name: "array_index_negative", input: "[1, 2, 3][-1]", output: "" },
    Case { name: "len_array_with_mixed_elements", input: r#"len([1, 2 * 2, 3 + 3, "hello"])"#, output: "4" },
    Case { name: "array_first", input: "first([1, 2, 3])", output: "1" },
    Case { name: "array_first_empty", input: "first([])", output: "" },
    Case { name: "array_last", input: "last([1, 2, 3])", output: "3" },
    Case { name: "array_last_empty", input: "last([])", output: "" },
    Case { name: "array_rest", input: "rest([1, 2, 3])", output: "[2, 3]" },
    Case { name: "array_rest_single", input: "rest([1])", output: "[]" },
    Case { name: "array_rest_chained", input: "rest(rest(rest(rest([1, 2, 3, 4, 5]))))", output: "[5]" },
    Case { name: "array_push", input: "push([1, 2], 3)", output: "[1, 2, 3]" },
    Case { name: "array_push_empty", input: "push([], 1)", output: "[1]" },

    // hash
    Case { name: "hash_index_hit", input: r#"{"foo": 5}["foo"]"#, output: "5" },
    Case { name: "hash_index_miss", input: r#"{"foo": 5}["bar"]"#, output: "" },
    Case { name: "hash_index_via_var", input: r#"let key = "foo"; {"foo": 5}[key]"#, output: "5" },
    Case { name: "hash_index_empty_hash", input: r#"{}["foo"]"#, output: "" },
    Case { name: "hash_index_int_key", input: "{5: 5}[5]", output: "5" },
    Case { name: "hash_index_true_key", input: "{true: 5}[true]", output: "5" },
    Case { name: "hash_index_false_key", input: "{false: 5}[false]", output: "5" },
    Case { name: "hash_dot_access_hit", input: r#"{"foo": 5}.foo"#, output: "5" },
    Case { name: "hash_dot_access_miss", input: r#"{"foo": 5}.bar"#, output: "" },
    Case { name: "hash_dot_access_via_var", input: r#"let h = {"foo": 5}; h.foo"#, output: "5" },
    Case { name: "hash_dot_access_nested", input: r#"let h = {"a": {"b": 42}}; h.a.b"#, output: "42" },

    // print
    Case { name: "print_basic", input: r#"print("hello")"#, output: "hello" },
    Case { name: "println_basic", input: r#"println("hello")"#, output: "hello\n" },
    Case { name: "print_multiple_calls", input: r#"print("a"); print("b"); print("c")"#, output: "abc" },
    Case { name: "println_multiple_calls", input: r#"println("a"); println("b")"#, output: "a\nb\n" },
    Case { name: "print_multiple_args", input: r#"print("x", 42, true)"#, output: "x 42 true" },
    Case { name: "println_multiple_args", input: r#"println("x", 42, true)"#, output: "x 42 true\n" },
    Case { name: "print_println_mixed", input: r#"print("a"); println("b"); print("c")"#, output: "ab\nc" },
    Case {
        name: "println_inside_function",
        input: r#"let greet = fn(name) { println("hi " + name) }; greet("bob")"#,
        output: "hi bob\n",
    },
    Case { name: "print_no_args", input: "print()", output: "" },

    // for_loops
    Case { name: "for_array_prints_each", input: "for x in [1, 2, 3] { print(x) }", output: "123" },
    Case { name: "for_array_of_strings", input: r#"for x in ["a", "b"] { println(x) }"#, output: "a\nb\n" },
    Case { name: "for_empty_array", input: "for x in [] { print(x) }", output: "" },
    Case { name: "for_named_array", input: "let nums = [10, 20]; for n in nums { print(n) }", output: "1020" },
    Case {
        name: "for_return_propagates",
        input: "let f = fn() { for x in [1, 2, 3] { if (x == 2) { return x } } }; f()",
        output: "2",
    },
    Case {
        name: "for_variable_is_scoped",
        input: "for x in [1, 2, 3] { x }; x",
        output: "ERROR: identifier not found: x",
    },
    Case { name: "for_over_non_iterable", input: "for x in 5 { x }", output: "ERROR: Integer is not iterable" },
    Case {
        name: "for_array_wrong_variable_count",
        input: "for a, b in [1, 2] { a }",
        output: "ERROR: for loop over Array expects 1 variable, got 2",
    },
    Case {
        name: "for_hash_wrong_variable_count",
        input: "for a, b, c in {1: 2} { a }",
        output: "ERROR: for loop over Hash expects 1 or 2 variables, got 3",
    },
    Case {
        name: "for_hash_key_and_value",
        input: r#"for k, v in {"a": 1} { print(k); print(v) }"#,
        output: "a1",
    },
    Case { name: "for_hash_key_only", input: r#"for k in {"only": 9} { print(k) }"#, output: "only" },
    Case { name: "for_empty_hash", input: "for k, v in {} { print(k) }", output: "" },
    Case {
        name: "for_hash_return_propagates",
        input: r#"
let find = fn(m, target) {
    for k, v in m {
        if (k == target) {
            return v
        }
    }
};

find({1: 10, 2: 20, 3: 30}, 2)"#,
        output: "20",
    },

    // floats
    Case { name: "float_literal", input: "3.5", output: "3.5" },
    Case { name: "float_negative_literal", input: "-2.25", output: "-2.25" },
    Case { name: "float_add", input: "1.5 + 2.25", output: "3.75" },
    Case { name: "float_add_via_vars", input: "let a = 1.5; let b = 2.75; a + b", output: "4.25" },
    Case { name: "float_array", input: "[1.5, 2.0 * 2.0, 3.0 + 0.25]", output: "[1.5, 4, 3.25]" },

    // null
    Case { name: "null_literal", input: "null", output: "" },
    Case { name: "null_via_var", input: "let x = null; x", output: "" },
    Case { name: "null_returned_from_function", input: "let f = fn() { null }; f()", output: "" },
    Case { name: "null_missing_hash_key", input: r#"{"a": 1}["b"]"#, output: "" },
    Case { name: "null_explicit_hash_value", input: r#"{"a": null}["a"]"#, output: "" },
    Case { name: "null_missing_dot_access", input: r#"let m = {"a": 1}; m.b"#, output: "" },
    Case { name: "null_array_out_of_bounds", input: "[1, 2, 3][5]", output: "" },
    Case { name: "null_inside_array", input: "[null][0]", output: "" },
    Case { name: "null_eq_null", input: "null == null", output: "true" },
    Case { name: "null_neq_null", input: "null != null", output: "false" },
    Case { name: "null_eq_int", input: "null == 5", output: "false" },
    Case { name: "int_eq_null", input: "5 == null", output: "false" },
    Case { name: "null_neq_int", input: "null != 5", output: "true" },
    Case { name: "int_neq_null", input: "5 != null", output: "true" },
    Case { name: "null_eq_string", input: r#"null == "a""#, output: "false" },
    Case { name: "null_eq_bool", input: "null == true", output: "false" },
    Case { name: "null_neq_bool", input: "null != false", output: "true" },
    Case { name: "null_var_eq_null", input: "let x = null; x == null", output: "true" },
    Case { name: "null_eq_null_var", input: "let x = null; null == x", output: "true" },
    Case { name: "null_var_neq_null", input: "let x = null; x != null", output: "false" },
    Case { name: "int_var_eq_null", input: "let y = 5; y == null", output: "false" },
    Case { name: "int_var_neq_null", input: "let y = 5; y != null", output: "true" },
    Case { name: "null_var_eq_null_var", input: "let a = null; let b = null; a == b", output: "true" },
    Case { name: "null_missing_hash_key_eq_null", input: r#"{"a": 1}["b"] == null"#, output: "true" },
    Case { name: "null_explicit_hash_value_eq_null", input: r#"{"a": null}["a"] == null"#, output: "true" },
    Case { name: "present_hash_value_neq_null", input: r#"{"a": 1}["a"] == null"#, output: "false" },
    Case { name: "null_array_out_of_bounds_eq_null", input: "[1, 2, 3][5] == null", output: "true" },
    Case {
        name: "null_is_falsy_in_if",
        input: r#"if (null) { print("t") } else { print("f") }"#,
        output: "f",
    },
    Case {
        name: "null_eq_check_in_if",
        input: r#"let x = null; if (x == null) { print("missing") }"#,
        output: "missing",
    },
];

#[test]
fn eval_table() {
    let mut failures = vec![];

    for case in CASES {
        let got = run(case.input);
        if got != case.output {
            failures.push(format!(
                "{}: input {:?}\n  expected: {:?}\n  received: {:?}",
                case.name, case.input, case.output, got
            ));
        }
    }

    if !failures.is_empty() {
        panic!("{} case(s) failed:\n{}", failures.len(), failures.join("\n"));
    }
}
