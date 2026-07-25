import init, {
  Interpreter,
  tokenize,
  Category,
} from "./lib/abclang/abclang";

await init();

export { Interpreter, tokenize, Category };
