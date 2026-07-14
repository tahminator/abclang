import React, { useCallback, useMemo } from "react";

import { useURLState } from ".";

export function useUrlEditorState(): [
  string,
  React.Dispatch<React.SetStateAction<string>>,
] {
  const [encoded, setEncoded] = useURLState<string>("state", "");

  const decoded = useMemo(() => decode(encoded), [encoded]);

  const setDecoded: React.Dispatch<React.SetStateAction<string>> = useCallback(
    (value) => {
      setEncoded((prev) => {
        const next =
          typeof value === "function"
            ? (value as (prev: string) => string)(decode(prev))
            : value;
        return encode(next);
      });
    },
    [setEncoded],
  );

  return [decoded, setDecoded];
}

function encode(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function decode(value: string): string {
  if (value === "") {
    return "";
  }

  try {
    const binary = atob(value);
    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch {
    return "";
  }
}
