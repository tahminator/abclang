import { Interpreter } from "../wasm";

export function run(source: string): string {
  const interpreter = new Interpreter();
  try {
    return interpreter.evaluate(source);
  } finally {
    interpreter.free();
  }
}
