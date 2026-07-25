import { Interpreter } from "../wasm";

type Result = string;
type Latency = string;

export function run(source: string): [Result, Latency] {
  const interpreter = new Interpreter();
  const start = performance.now();
  try {
    const result = interpreter.evaluate(source);
    const end = performance.now();
    return [result, (end - start).toFixed(4)];
  } finally {
    interpreter.free();
  }
}
