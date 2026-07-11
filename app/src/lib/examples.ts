export type Example = {
  name: string;
  code: string;
};

export const examples: Example[] = [
  {
    name: "Arithmetic",
    code: `// fnteger arithmetic testing usual precedence rules.
let a = 5;
let b = 10;

(a + b) * 2 - a / 5;
`,
  },
  {
    name: "Conditionals",
    code: `// if / else is an expression: it evaluates to a value.
let max = fn(a, b) {
  if (a > b) {
    return a;
  } else {
    return b;
  }
};

max(21, 7);
`,
  },
  {
    name: "Functions",
    code: `// functions are first-class values bound with let.
let double = fn(x) { x * 2 };
let apply = fn(f, x) { f(x) };

apply(double, 16);
`,
  },
  {
    name: "Closures",
    code: `// inner functions capture their surrounding environment.
let newAdder = fn(x) {
  fn(y) { x + y };
};

let addTwo = newAdder(2);
addTwo(2);
`,
  },
  {
    name: "Recursion",
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

export const defaultExample = examples[0];
