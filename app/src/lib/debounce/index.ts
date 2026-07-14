import { useCallback, useEffect, useRef, useState } from "react";

export interface UseDebouncedValueOptions {
  leading?: boolean;
}

export interface UseDebouncedValueHandlers {
  cancel: () => void;
  flush: () => void;
}

export type UseDebouncedValueReturnValue<T> = [
  T,
  () => void,
  UseDebouncedValueHandlers,
];

// https://github.com/mantinedev/mantine/blob/9dda4cac3da8eab49b9d60181d81a11778223307/packages/@mantine/hooks/src/use-debounced-value/use-debounced-value.ts
export function useDebouncedValue<T = unknown>(
  value: T,
  wait: number,
  options: UseDebouncedValueOptions = { leading: false },
): UseDebouncedValueReturnValue<T> {
  const [_value, setValue] = useState(value);
  const mountedRef = useRef(false);
  const timeoutRef = useRef<number | null>(null);
  const cooldownRef = useRef(false);

  const latestValueRef = useRef(value);
  // eslint-disable-next-line react-hooks/refs
  latestValueRef.current = value;

  const cancel = useCallback(() => {
    window.clearTimeout(timeoutRef.current!);
    timeoutRef.current = null;
    cooldownRef.current = false;
  }, []);

  const flush = useCallback(() => {
    if (timeoutRef.current) {
      cancel();
      cooldownRef.current = false;
      setValue(latestValueRef.current);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (mountedRef.current) {
      if (!cooldownRef.current && options.leading) {
        cooldownRef.current = true;
        setValue(value);
        timeoutRef.current = window.setTimeout(() => {
          cooldownRef.current = false;
        }, wait);
      } else {
        cancel();
        timeoutRef.current = window.setTimeout(() => {
          cooldownRef.current = false;
          setValue(value);
        }, wait);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value, options.leading, wait]);

  useEffect(() => {
    mountedRef.current = true;
    return cancel;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return [_value, cancel, { cancel, flush }];
}

// eslint-disable-next-line @typescript-eslint/no-namespace
export namespace useDebouncedValue {
  export type Handlers = UseDebouncedValueHandlers;
  export type Options = UseDebouncedValueOptions;
  export type ReturnValue<T> = UseDebouncedValueReturnValue<T>;
}
