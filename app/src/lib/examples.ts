export type Example = {
  name: string;
  code: string;
};

export const examples: Example[] = [
  {
    name: "strings",
    code: `// string concatenation
let firstname = "johnny";
let middlename = "moneybaggs";
let lastname = "appleseed";

let fullname = firstname + " " + middlename[0] + " " + lastname;
fullname
`,
  },
  {
    name: "arithmetic",
    code: `// usual precedence rules
let a = 5;
let b = 10;

(a + b) * 2 - a / 5;
`,
  },
  {
    name: "reassignment",
    code: `// '=' reassigns an existing binding
let count = 0;
count = count + 1;
count = count + 1;
println("count:", count);

let nums = [1, 2, 3];
nums[0] = 99;
nums[2] = nums[2] + 100;
println("nums:", nums);

let ages = {"alice": 30};
ages["bob"] = 25;
ages["alice"] = 31;
ages.carol = 41;
println("ages:", ages);

let grid = [[0, 0], [0, 0]];
grid[1][0] = 7;
println("grid:", grid);
`,
  },
  {
    name: "numbers",
    code: `// ints and floats
let count = 42;
let negative = -7;
let pi = 3.14159;
let tiny = -0.5;

println("int:", count);
println("float:", pi);
println("int math:", (count + 8) * 2 - 10 / 5);
println("float math:", pi * 2.0);
println("mixed:", 3 + 0.5);

// int/int truncates, float/float doesn't
println("int division:", 7 / 2);
println("float division:", 7.0 / 2.0);

let mixed = [1, 2.5, 3, -4.25, count, pi];
println("array:", mixed);
println("first + last:", mixed[0] + mixed[len(mixed) - 1]);

let constants = {"pi": 3.14159, "e": 2.71828, "answer": 42};
println("hashmap:", constants);
println("e:", constants["e"]);
`,
  },
  {
    name: "conditionals",
    code: `// if/else is an expression
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
    code: `// arrays can hold mixed types
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

abc = push(abc, 3)
println(abc)

abc[0] = 100
println(abc[0])
`,
  },
  {
    name: "hashmaps",
    code: `// keys can be int, bool, or string
let people = [{"name": "Alice", "age": 24}, {"name": "Anna", "age": 28}];

people[1]["name"] = "Beth";
people[0]["age"] = people[0]["age"] + 1;

people[1]["name"] + " & " + people[0]["name"];
`,
  },
  {
    name: "dot syntax / classes",
    code: `// dot syntax: hash.field is sugar for hash["field"]
let person = {"name": "Alice", "age": 24};
println(person.name, person.age);

// real classes: self.field sets instance state
class Point {
  fn new(self, x, y) {
    self.x = x;
    self.y = y;
  }

  fn dist(self, other) {
    let dx = self.x - other.x;
    let dy = self.y - other.y;
    dx * dx + dy * dy
  }

  // no self = static
  // can be called on class or an instance of class
  fn origin() { Point(0, 0) }
}

let a = Point.origin();
let b = Point(3, 4);
println(a.dist(b));
`,
  },
  {
    name: "builtins",
    code: `// len() = length of a string or array
let firstname = "johnny";
let lastname = "appleseed";
let fullname = firstname + " " + lastname;

// max()/min() compare two ints
let longest = max(len(firstname), len(lastname));
let shortest = min(len(firstname), len(lastname));
println("minmax example:", [len(fullname), longest, shortest]);

// first(arr) -> arr[0]
// last(arr) -> arr[len(arr) - 1]
// rest(arr) -> arr[1..]
// push(arr, itm) -> arr + [itm]
// range(n) -> [0..n), range(start, end) -> [start..end)
println("push array example:", push([1, 2], 3));
println("range example:", range(1, 5));

// chain them into a map()
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

// and a reduce()
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
    code: `// functions are first-class values
let double = fn(x) { x * 2 };
let apply = fn(f, x) { f(x) };

apply(double, 16);
`,
  },
  {
    name: "closures",
    code: `// closures capture their environment
let newAdder = fn(x) {
  fn(y) { x + y };
};

let addTwo = newAdder(2);
addTwo(2);
`,
  },
  {
    name: "recursion",
    code: `// functions can call themselves
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

// range pairs with for loop to iterate by index
let nums = [10, 20, 30];
for i in range(len(nums)) {
  println(i, nums[i]);
}

// or iterate arrays directly
for n in nums {
  println(n);
}
println("");

// or iterate maps via \`for k, v in map {  }\`
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
  {
    name: "Leetcode - 91. Decode Ways",
    code: `// https://leetcode.com/problems/decode-ways/
// C++ solution: https://codebloom.patinanetwork.org/submission/f2bda114-ce01-4f57-96db-1d251263fd59
let decodeWays = fn(s) {
  let cache = { len(s): 1 }

  let dp = fn(i) {
    if (cache[i] != null) {
      return cache[i];
    }

    if (s[i] == '0') {
      return 0;
    }

    let res = dp(i + 1);
    if (i + 1 < len(s)) {
      if (s[i] == '1') {
        res = res + dp(i + 2);
      } else {
        if (s[i] == '2') {
          for n in "0123456" {
            if (s[i + 1] == n) {
              res = res + dp(i + 2);
            }
          }
        }
      }
    }
    cache[i] = res;
    return res;
  }

  return dp(0);
}

println("expected", 2, "received", decodeWays("11106"));
println("expected", 2, "received", decodeWays("12"));
println("expected", 3, "received", decodeWays("226"));
println("expected", 0, "received", decodeWays("06"));
println("expected", 1836311903, "received", decodeWays("111111111111111111111111111111111111111111111"));
`,
  },
];
