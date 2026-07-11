import { useState } from "react";
import { run } from "../lib/interpreter";
import { defaultExample, examples } from "../lib/examples";

export type RunResult = {
  output: string;
  hasRun: boolean;
};

export function useEditor() {
  const [code, setCode] = useState(defaultExample.code);
  const [selected, setSelected] = useState(defaultExample.name);
  const [result, setResult] = useState<RunResult>({
    output: "",
    hasRun: false,
  });

  const onSelectExample = (name: string) => {
    const example = examples.find((e) => e.name === name);
    if (!example) return;
    setSelected(name);
    setCode(example.code);
    setResult({ output: "", hasRun: false });
  };

  const onChange = (next: string) => {
    setCode(next);
    setSelected("");
  };

  const onRun = () => {
    setResult({ output: run(code), hasRun: true });
  };

  return { code, selected, result, examples, onSelectExample, onChange, onRun };
}
