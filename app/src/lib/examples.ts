export type Example = {
  name: string;
  code: string;
};

export const examples: Example[] = [
  {
    name: "strings",
    code: `// strings are supported in abclang, along with string concatenation
let firstname = "johnny";
let middlename = "moneybaggs";
let lastname = "appleseed";

let fullname = firstname + " " + middlename[0] + " " + lastname;
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
    name: "reassignment",
    code: `// 'let' introduces a binding; a bare '=' reassigns an existing one.
let count = 0;
count = count + 1;
count = count + 1;
println("count:", count); // => 2

// arrays are mutable: assign straight into an index (must be in bounds).
let nums = [1, 2, 3];
nums[0] = 99;
nums[2] = nums[2] + 100;
println("nums:", nums); // => [99, 2, 103]

// hashmaps too: assigning a new key inserts, an existing key updates.
let ages = {"alice": 30};
ages["bob"] = 25; // insert
ages["alice"] = 31; // update
ages.carol = 41; // dot sugar for ages["carol"] = 41
println("ages:", ages);

// assignment reaches through nesting and shared references.
let grid = [[0, 0], [0, 0]];
grid[1][0] = 7;
println("grid:", grid); // => [[0, 0], [7, 0]]
`,
  },
  {
    name: "numbers",
    code: `// abclang has two number types: ints and floats.

// integer literals
let count = 42;
let negative = -7;

// float literals (any number with a decimal point)
let pi = 3.14159;
let tiny = -0.5;

// printing numbers
println("int:", count);
println("float:", pi);

// arithmetic follows the usual precedence rules
println("int math:", (count + 8) * 2 - 10 / 5);
println("float math:", pi * 2.0);

// mixing ints and floats promotes the result to a float
println("mixed:", 3 + 0.5);

// division: int / int stays an int (truncates), float division keeps the remainder
println("int division:", 7 / 2);
println("float division:", 7.0 / 2.0);

// numbers can live side by side in arrays
let mixed = [1, 2.5, 3, -4.25, count, pi];
println("array:", mixed);
println("first + last:", mixed[0] + mixed[len(mixed) - 1]);

// and as hashmap values
let constants = {"pi": 3.14159, "e": 2.71828, "answer": 42};
println("hashmap:", constants);
println("e:", constants["e"]);
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

// use push() to grow the array with a new element
abc = push(abc, 3)
println(abc)

// overwrite a specific index in place
abc[0] = 100
println(abc[0])
`,
  },
  {
    name: "hashmaps",
    code: `// abclang supports hashmaps. they do not have to be homogoneous (same elements). you can use integer, boolean, or string as key.
let people = [{"name": "Alice", "age": 24}, {"name": "Anna", "age": 28}];

// index and key assignment mutate in place, and reach through nesting.
people[1]["name"] = "Beth";
people[0]["age"] = people[0]["age"] + 1;

people[1]["name"] + " & " + people[0]["name"];
`,
  },
  {
    name: "dot syntax / classes",
    code: `// hashmap fields can be read with dot syntax: hash.field is just
// shorthand for hash["field"]. the name after the dot is always a
// literal key (not a variable).
let person = {"name": "Alice", "age": 24};

println(person.name);
println(person.age);

// abclang has no classes, but you can model them with hashmaps:
// store data under keys and behavior as fn values. because functions
// are closures, a constructor captures its arguments, so the methods
// can use the fields without any "this".
let newRect = fn(width, height) {
  {
    "width": width,
    "height": height,
    "area": fn() { width * height },
    // methods can even return new "instances"
    "scale": fn(factor) { newRect(width * factor, height * factor) }
  }
};

let r = newRect(3, 4);
println("width:", r.width);
println("area:", r.area());

let big = r.scale(2);
println("scaled width:", big.width);
println("scaled area:", big.area());
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
// push(arr, itm) -> grows arr = [...arr, itm]
// range(n) -> [0, 1, ..., n - 1]
// range(start, end) -> [start, ..., end - 1]

// to overwrite in place, assign straight into an index or key:
// arr[idx] = val (idx must be in bounds), map[key] = val (inserts or updates)

println("push array example:", push([1, 2], 3));
println("range example:", range(1, 5));

// you can chain these together to make a map function!
let map = fn(arr, f) {
    let iter = fn(arr, accumulated) {
        if (len(arr) == 0) {
            accumulated
        } else {
            iter(rest(arr), push(accumulated, f(first(arr))));
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
    name: "iterators / range",
    code: `// range(n) builds [0, 1, ..., n - 1]
// range(start, end) builds [start, ..., end - 1]
println("range(n):", range(3));
println("range(start, end):", range(2, 6));

// range pairs nicely with a for loop to iterate by index
let nums = [10, 20, 30];
for i in range(len(nums)) {
  println(i, nums[i]);
}

// for loops iterate arrays directly too
for n in nums {
  println(n);
}
println("");

// maps are iterable with a "key, value" for loop
let ages = {"alice": 30, "bob": 25};
for name, age in ages {
  println(name, age);
}
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
    seen[nums[i]] = i;
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
