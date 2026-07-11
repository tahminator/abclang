import type { RunResult } from "../../hooks/editor";

export type OutputPanelProps = {
  result: RunResult;
};

export function OutputPanel({ result }: OutputPanelProps) {
  return (
    <div className="panel">
      <div className="panel__header">Output</div>
      <pre className={result.hasRun ? "output" : "output output--empty"}>
        {result.hasRun ? result.output : "Press Run to execute your program."}
      </pre>
    </div>
  );
}
