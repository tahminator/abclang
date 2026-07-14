import type { Example } from "../../lib/examples";

export type ToolbarProps = {
  examples: Example[];
  selected: string;
  onSelectExample: (name: string) => void;
  onRun: () => void;
  onClear: () => void;
};

export function Toolbar({
  examples,
  selected,
  onSelectExample,
  onRun,
  onClear,
}: ToolbarProps) {
  return (
    <div className="toolbar">
      <select
        className="select"
        value={selected}
        onChange={(e) => onSelectExample(e.target.value)}
      >
        <option value="" disabled>
          Examples…
        </option>
        {examples.map((ex) => (
          <option key={ex.name} value={ex.name}>
            {ex.name}
          </option>
        ))}
      </select>

      <div className="buttons">
        <button className="button run-button" onClick={onRun}>
          ▶ Run
        </button>
        <button className="button clear-button" onClick={onClear}>
          Clear
        </button>
      </div>
    </div>
  );
}
