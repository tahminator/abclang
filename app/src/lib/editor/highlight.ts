import { RangeSetBuilder } from "@codemirror/state";
import {
  Decoration,
  ViewPlugin,
  EditorView,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import { tokenize, Category } from "../../wasm";

const CATEGORY_CLASS: Record<Category, string> = {
  [Category.Keyword]: "cm-abc-keyword",
  [Category.Number]: "cm-abc-number",
  [Category.String]: "cm-abc-string",
  [Category.Operator]: "cm-abc-operator",
  [Category.Punctuation]: "cm-abc-punctuation",
  [Category.Ident]: "cm-abc-ident",
  [Category.Comment]: "cm-abc-comment",
  [Category.Illegal]: "cm-abc-illegal",
};
const highlightTheme = EditorView.theme({
  ".cm-abc-keyword": { color: "#c678dd" },
  ".cm-abc-number": { color: "#d19a66" },
  ".cm-abc-string": { color: "#98c379" },
  ".cm-abc-operator": { color: "#56b6c2" },
  ".cm-abc-punctuation": { color: "var(--text-muted)" },
  ".cm-abc-ident": { color: "#61afef" },
  ".cm-abc-comment": { color: "var(--text-muted)", fontStyle: "italic" },
  ".cm-abc-illegal": { color: "#e06c75", textDecoration: "underline wavy" },
});

const MARKS = new Map<number, Decoration>(
  Object.entries(CATEGORY_CLASS).map(([cat, cls]) => [
    Number(cat),
    Decoration.mark({ class: cls }),
  ]),
);

function buildDecorations(view: EditorView): DecorationSet {
  const tokens = tokenize(view.state.doc.toString());
  const spans: Array<[number, number, number]> = [];
  for (let i = 0; i + 2 < tokens.length; i += 3) {
    spans.push([tokens[i], tokens[i + 1], tokens[i + 2]]);
  }
  spans.sort((a, b) => a[0] - b[0]);

  const builder = new RangeSetBuilder<Decoration>();
  for (const [start, end, category] of spans) {
    const mark = MARKS.get(category);
    if (mark && end > start) {
      builder.add(start, end, mark);
    }
  }
  return builder.finish();
}

const highlightPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged) {
        this.decorations = buildDecorations(update.view);
      }
    }
  },
  { decorations: (plugin) => plugin.decorations },
);

export function abclangHighlight() {
  return [highlightPlugin, highlightTheme];
}
