export type CodePanelProps = {
  code: string;
  onChange: (value: string) => void;
  onRun: () => void;
};

export function CodePanel({ code, onChange, onRun }: CodePanelProps) {
  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      onRun();
      return;
    }

    if (e.key === "Tab") {
      e.preventDefault();
      const el = e.currentTarget;
      const { selectionStart: start, selectionEnd: end } = el;
      onChange(code.slice(0, start) + "  " + code.slice(end));
      requestAnimationFrame(() => {
        el.selectionStart = el.selectionEnd = start + 2;
      });
    }
  };

  return (
    <div className="panel">
      <textarea
        className="code-input"
        value={code}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        spellCheck={false}
        autoComplete="off"
        autoCapitalize="off"
        autoCorrect="off"
      />
    </div>
  );
}
