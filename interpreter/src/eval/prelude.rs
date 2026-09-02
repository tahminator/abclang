use crate::{
    eval::{evaluate, object::environment::Env},
    lexer::Lexer,
    parser::Parser,
};

const PRELUDE_SRC: &str = r#"
class Array {
    // Appends item to the end of the array.
    fn push(self, item) {
        return __INTERNALS_array_push(self, item);
    }

    // Removes the last item from the array.
    // It returns the removed item as the result.
    fn pop(self) {
        return __INTERNALS_array_pop(self);
    }

    // Returns the first item of the array as the result.
    fn first(self) {
        return __INTERNALS_array_first(self);
    }

    // Returns the last item of the array as the result.
    fn last(self) {
        return __INTERNALS_array_last(self);
    }

    // Returns a new array as the result.
    // The new array has all items of the array except the first item.
    fn rest(self) {
        return __INTERNALS_array_rest(self);
    }

    // Returns the number of items in the array as the result.
    fn len(self) {
        return __INTERNALS_array_len(self);
    }

    // Applies the function F to each item of the array.
    // It returns a new array with the results as the result.
    fn map(self, f) {
        let result = [];
        for x in self {
            __INTERNALS_array_push(result, f(x));
        }
        return result;
    }

    // Returns the items for which the function F returns a true result.
    // The returned items are allocated to a new array.
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
    // Returns the value that is related to key as the result.
    // Will return null if key is not in the hashmap.
    fn get(self, key) {
        return self[key];
    }

    // Sets the value that is related to key to VALUE.
    // Returns hashmap as result.
    fn set(self, key, value) {
        self[key] = value;
        return self;
    }

    // Will output boolean determining if key is in the hashmap.
    fn has(self, key) {
        return __INTERNALS_hash_has(self, key);
    }

    // Removes key from the hashmap.
    // Will return the removed value.
    fn remove(self, key) {
        return __INTERNALS_hash_remove(self, key);
    }

    // Returns a new array with all the keys in the hashmap.
    fn keys(self) {
        return __INTERNALS_hash_keys(self);
    }

    // Returns a new array with all the values in the hashmap.
    fn values(self) {
        return __INTERNALS_hash_values(self);
    }

    // Returns the number of key-value pairs in the hashmap.
    fn len(self) {
        return __INTERNALS_hash_len(self);
    }
}
"#;

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
