import CodeMirror from "@uiw/react-codemirror";
import { EditorView, keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { abclangHighlight } from "../../lib/editor/highlight";

export type CodePanelProps = {
  code: string;
  onChange: (value: string) => void;
  onRun: () => void;
};

const editorTheme = EditorView.theme(
  {
    "&": { height: "100%", background: "var(--bg)", color: "var(--text)" },
    ".cm-scroller": { overflow: "auto", fontFamily: "inherit" },
    ".cm-content": { padding: "12px 0", caretColor: "#ffffff" },
    "&.cm-focused": { outline: "none" },
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "#ffffff" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
      { backgroundColor: "rgba(88, 166, 255, 0.45)" },
    ".cm-gutters": {
      background: "var(--bg)",
      color: "var(--text-muted)",
      border: "none",
    },
  },
  { dark: true },
);

export function CodePanel({ code, onChange, onRun }: CodePanelProps) {
  return (
    <CodeMirror
      className="panel"
      value={code}
      height="100%"
      theme="none"
      onChange={onChange}
      extensions={[
        keymap.of([
          {
            key: "Mod-Enter",
            run: () => {
              onRun();
              return true;
            },
          },
          indentWithTab,
        ]),
        abclangHighlight(),
        editorTheme,
      ]}
    />
  );
}
