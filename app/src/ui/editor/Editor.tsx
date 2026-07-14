import { useEditor } from "../../hooks/editor";
import { CodePanel } from "./CodePanel";
import { OutputPanel } from "./OutputPanel";
import { Toolbar } from "./Toolbar";

export function Editor() {
  const {
    code,
    selected,
    result,
    examples,
    onSelectExample,
    onChange,
    onRun,
    onClear,
  } = useEditor();

  return (
    <div className="editor">
      <Toolbar
        examples={examples}
        selected={selected}
        onSelectExample={onSelectExample}
        onRun={onRun}
        onClear={onClear}
      />
      <div className="panels">
        <CodePanel code={code} onChange={onChange} onRun={onRun} />
        <OutputPanel result={result} />
      </div>
    </div>
  );
}
