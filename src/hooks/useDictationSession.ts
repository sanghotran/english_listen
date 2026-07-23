import { useCallback, useState } from "react";
import { computeAccuracy, diffWords, type WordDiffToken } from "../utils/diff";

export interface DictationResult {
  tokens: WordDiffToken[];
  accuracy: number;
}

/** Manages a single dictation attempt: typed input state + diff result against a reference transcript. */
export function useDictationSession(reference: string) {
  const [input, setInput] = useState("");
  const [result, setResult] = useState<DictationResult | null>(null);

  const submit = useCallback(() => {
    const tokens = diffWords(input, reference);
    const accuracy = computeAccuracy(tokens);
    const next = { tokens, accuracy };
    setResult(next);
    return next;
  }, [input, reference]);

  const reset = useCallback(() => {
    setInput("");
    setResult(null);
  }, []);

  return { input, setInput, result, submit, reset };
}
