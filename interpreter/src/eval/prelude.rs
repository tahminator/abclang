use crate::{
    eval::{evaluate, object::environment::Env},
    lexer::Lexer,
    parser::Parser,
};

/// The abclang standard library, written in abclang itself. Loaded into every fresh
/// root [`Environment`](super::object::environment::Environment) so that primitive
/// types like `Array` and `Hash` get their methods (`.push`, `.map`, `.get`, ...) from
/// real classes rather than hardcoded interpreter logic. Methods that can't be
/// expressed in abclang alone (raw vec/map mutation) delegate to the `__INTERNALS_`
/// builtins in [`crate::eval::builtins`].
const PRELUDE_SRC: &str = r#"
class Array {
    // Adds ITEM to the end of the array.
    fn push(self, item) {
        return __INTERNALS_array_push(self, item);
    }

    // Removes the last item from the array.
    // It gives the removed item as the result.
    fn pop(self) {
        return __INTERNALS_array_pop(self);
    }

    // Gives the first item of the array as the result.
    fn first(self) {
        return __INTERNALS_array_first(self);
    }

    // Gives the last item of the array as the result.
    fn last(self) {
        return __INTERNALS_array_last(self);
    }

    // Gives a new array as the result.
    // The new array has all items of the array except the first item.
    fn rest(self) {
        return __INTERNALS_array_rest(self);
    }

    // Gives the number of items in the array as the result.
    fn len(self) {
        return __INTERNALS_array_len(self);
    }

    // Applies the function F to each item of the array.
    // It gives a new array with the results as the result.
    fn map(self, f) {
        let result = [];
        for x in self {
            __INTERNALS_array_push(result, f(x));
        }
        return result;
    }

    // Keeps the items for which the function F gives a true result.
    // It gives a new array with these items as the result.
    fn filter(self, f) {
        let result = [];
        for x in self {
            if (f(x)) {
                __INTERNALS_array_push(result, x);
            }
        }
        return result;
    }
}

class HashMap {
    // Gives the value that is related to KEY as the result.
    // It gives NULL if KEY is not in the hash.
    fn get(self, key) {
        return self[key];
    }

    // Sets the value that is related to KEY to VALUE.
    // It gives the hash as the result.
    fn set(self, key, value) {
        self[key] = value;
        return self;
    }

    // Tells if KEY is in the hash.
    fn has(self, key) {
        return __INTERNALS_hash_has(self, key);
    }

    // Removes KEY from the hash.
    // It gives the removed value as the result.
    fn remove(self, key) {
        return __INTERNALS_hash_remove(self, key);
    }

    // Gives a new array with all the keys in the hash.
    fn keys(self) {
        return __INTERNALS_hash_keys(self);
    }

    // Gives a new array with all the values in the hash.
    fn values(self) {
        return __INTERNALS_hash_values(self);
    }

    // Gives the number of key-value pairs in the hash.
    fn len(self) {
        return __INTERNALS_hash_len(self);
    }
}
"#;

/// Evaluates the stdlib prelude into `env`. Panics on failure since the prelude
/// source is fixed at compile time and any failure is an interpreter bug, not
/// something a caller can recover from.
pub fn load_to_env(env: &Env) {
    let lexer = Lexer::new(PRELUDE_SRC);

    let mut parser = Parser::new(lexer)
        .unwrap_or_else(|err| panic!("abclang stdlib prelude failed to lex: {err}"));

    let program = parser
        .parse_program()
        .unwrap_or_else(|errors| panic!("abclang stdlib prelude has parser error(s): {errors:?}"));

    evaluate(&program, env)
        .unwrap_or_else(|err| panic!("abclang stdlib prelude failed to evaluate: {err}"));
}
