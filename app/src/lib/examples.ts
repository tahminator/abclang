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
`,
  },
  {
    name: "builtins",
    code: `// abclang has some default builtins you may use

// len() can be called on any string to output total length of string
let firstname = "johnny";
let lastname = "appleseed";
let fullname = firstname + " " + lastname;

// max() and min() compare two integers
let longest = max(len(firstname), len(lastname));
let shortest = min(len(firstname), len(lastname));

[len(fullname), longest, shortest]
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
];
