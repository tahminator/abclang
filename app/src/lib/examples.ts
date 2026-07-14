export type Example = {
  name: string;
  code: string;
};

export const examples: Example[] = [
  {
    name: "strings",
    code: `// strings are supported in abclang, along with string concatenation
let firstname = "johnny";
let lastname = "appleseed";

let fullname = firstname + " " + lastname;
fullname
`,
  },
  {
    name: "arithmetic",
    code: `// fnteger arithmetic testing usual precedence rules.
let a = 5;
let b = 10;

(a + b) * 2 - a / 5;
`,
  },
  {
    name: "conditionals",
    code: `// if / else is an expression: it evaluates to a value.
let classify = fn(n) {
  if (n > 0) {
    return "positive";
  } else {
    return "non-positive";
  }
};

classify(21);
`,
  },
  {
    name: "arrays",
    code: `// abclang supports arrays. they do not have to be homogoneous (same elements).
let getOne = fn() { 1 };

let abc = [
  1,
  2 * 2,
  3 + 3,
  getOne(),
  max(3, 92),
  "jonny",
  "jonny appleseeds the name",
  len("hello world")
];

abc

// or print a specific index
// abc[0]
`,
  },
  {
    name: "hashmaps",
    code: `// abclang supports hashmaps. they do not have to be homogoneous (same elements). you can use integer, boolean, or string as key.
let people = [{"name": "Alice", "age": 24}, {"name": "Anna", "age": 28}];

people[1]["name"] + " & " + people[0]["name"];
`,
  },
  {
    name: "builtins",
    code: `// abclang has some default builtins you may use

// len() can be called on any string or array to output total length of string
let firstname = "johnny";
let lastname = "appleseed";
let fullname = firstname + " " + lastname;

// max() and min() compare two integers
let longest = max(len(firstname), len(lastname));
let shortest = min(len(firstname), len(lastname));

println("minmax example:", [len(fullname), longest, shortest]);

// abclang supports
// first(arr) -> arr[0]
// last(arr) -> arr[len(arr) - 1]
// rest(arr) -> returns arr[1..len(arr)]
// append(arr, itm) -> returns arr = [...arr, itm]
// append(map, key, val) -> returns map with map[key] = val set
// range(n) -> [0, 1, ..., n - 1]
// range(start, end) -> [start, ..., end - 1]

println("append array example:", append([1, 2], 3));
println("append map example:", append({"a": 1}, "b", 2));
println("range example:", range(1, 5));

// you can chain these together to make a map function!
let map = fn(arr, f) {
    let iter = fn(arr, accumulated) {
        if (len(arr) == 0) {
            accumulated
        } else {
            iter(rest(arr), append(accumulated, f(first(arr))));
        }
    };
    iter(arr, []);
};

let a = [1, 2, 3, 4];
let double = fn(x) { x * 2 };

println("map example:", map(a, double));

// you can chain these together to make a reduce & sum function!
let reduce = fn(arr, initial, f) {
  let iter = fn(arr, result) {
    if (len(arr) == 0) {
      result
    } else {
      iter(rest(arr), f(result, first(arr)));
    }
  };
  iter(arr, initial);
};

let sum = fn(arr) {
  reduce(arr, 0, fn(initial, el) { initial + el });
};

println("sum example:", sum([1, 2, 3, 4, 5]));
`,
  },
  {
    name: "functions",
    code: `// functions are first-class values bound with let.
let double = fn(x) { x * 2 };
let apply = fn(f, x) { f(x) };

apply(double, 16);
`,
  },
  {
    name: "closures",
    code: `// inner functions capture their surrounding environment.
let newAdder = fn(x) {
  fn(y) { x + y };
};

let addTwo = newAdder(2);
addTwo(2);
`,
  },
  {
    name: "recursion",
    code: `// a function can call itself through its binding.
let fib = fn(n) {
  if (n < 2) {
    return n;
  }
  fib(n - 1) + fib(n - 2);
};

fib(10);
`,
  },
  {
    name: "LeetCode - 1. Two Sum",
    code: `let twoSum = fn(nums, target) {
  let seen = {};
  for i in range(len(nums)) {
    let need = target - nums[i];
    if (seen[need]) {
      return [seen[need], i];
    }
    seen = append(seen, nums[i], i);
  }
  return [];
};

println(twoSum([2, 7, 11, 15], 9));
println(twoSum([3, 2, 4], 6));
println(twoSum([3, 3], 6));
println(twoSum([3,2,3], 6));
`,
  },
];
