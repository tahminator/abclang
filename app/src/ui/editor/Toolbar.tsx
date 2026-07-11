import type { Example } from "../../lib/examples";

export type ToolbarProps = {
  examples: Example[];
  selected: string;
  onSelectExample: (name: string) => void;
  onRun: () => void;
};

export function Toolbar({
  examples,
  selected,
  onSelectExample,
  onRun,
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

      <button className="run-button" onClick={onRun}>
        ▶ Run
      </button>
    </div>
  );
}
