import { useState } from "react";
import { run } from "../lib/interpreter";
import { examples } from "../lib/examples";
import { useUrlEditorState } from "../lib/url/editor";

export type RunResult = {
  output: string;
  hasRun: boolean;
  timeToRun?: string;
};

export function useEditor() {
  const [code, setCode] = useUrlEditorState();
  const [selected, setSelected] = useState("");
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

  const onClear = () => {
    setResult({ output: "", hasRun: false });
  };

  const onRun = () => {
    const [result, timeToRun] = run(code);
    setResult({ output: result, timeToRun, hasRun: true });
  };

  return {
    code,
    selected,
    result,
    examples,
    onSelectExample,
    onChange,
    onRun,
    onClear,
  };
}
