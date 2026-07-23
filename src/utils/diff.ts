import { diffArrays } from "diff";

export type WordDiffStatus = "correct" | "missing" | "extra";

export interface WordDiffToken {
  text: string;
  status: WordDiffStatus;
}

// A few contractions are irregular enough that the generic "stem + n't" rule below would mangle
// them (e.g. "can't" -> stem "ca" instead of "can", since "can" itself ends in "n").
const IRREGULAR_CONTRACTIONS: Array<[RegExp, string]> = [
  [/\bwon't\b/gi, "will not"],
  [/\bshan't\b/gi, "shall not"],
  [/\bcan't\b/gi, "can not"],
];

/**
 * Expands unambiguous contractions to their full form before tokenizing, so "don't" typed as
 * "do not" (or vice versa) scores as correct — VOA transcripts use contractions, but a learner
 * may type either form. `'s`/`'d` are genuinely ambiguous (is/has, would/had) and are left alone.
 */
function expandContractions(text: string): string {
  let result = text;
  for (const [pattern, replacement] of IRREGULAR_CONTRACTIONS) {
    result = result.replace(pattern, replacement);
  }
  return result
    .replace(/\b(\w+)n't\b/gi, "$1 not")
    .replace(/\b(\w+)'re\b/gi, "$1 are")
    .replace(/\b(\w+)'ve\b/gi, "$1 have")
    .replace(/\b(\w+)'ll\b/gi, "$1 will")
    .replace(/\bI'm\b/gi, "I am");
}

function tokenize(text: string): string[] {
  return expandContractions(text).trim().split(/\s+/).filter(Boolean);
}

function normalize(word: string): string {
  return word.toLowerCase().replace(/^[^\p{L}\p{N}'-]+|[^\p{L}\p{N}'-]+$/gu, "");
}

/**
 * Word-level diff between what the user typed and the reference transcript.
 * "missing" = present in the transcript but absent from the user's input.
 * "extra" = present in the user's input but absent from the transcript.
 */
export function diffWords(input: string, reference: string): WordDiffToken[] {
  const refTokens = tokenize(reference);
  const inTokens = tokenize(input);

  const changes = diffArrays(refTokens, inTokens, {
    comparator: (a, b) => normalize(a) === normalize(b),
  });

  const tokens: WordDiffToken[] = [];
  for (const change of changes) {
    if (change.removed) {
      for (const word of change.value) tokens.push({ text: word, status: "missing" });
    } else if (change.added) {
      for (const word of change.value) tokens.push({ text: word, status: "extra" });
    } else {
      for (const word of change.value) tokens.push({ text: word, status: "correct" });
    }
  }
  return tokens;
}

interface Run {
  status: WordDiffStatus;
  length: number;
}

function toRuns(tokens: WordDiffToken[]): Run[] {
  const runs: Run[] = [];
  for (const token of tokens) {
    const last = runs[runs.length - 1];
    if (last && last.status === token.status) last.length += 1;
    else runs.push({ status: token.status, length: 1 });
  }
  return runs;
}

/**
 * Accuracy = 1 - word error rate (standard ASR WER: substitutions+missing+extra / reference length).
 * A missing+extra run adjacent to each other is one substitution "edit" per word pair, not two
 * separate errors — otherwise a single wrong word would incorrectly count as 2 errors.
 */
export function computeAccuracy(tokens: WordDiffToken[]): number {
  const referenceCount = tokens.filter((t) => t.status !== "extra").length;
  if (referenceCount === 0) return 0;

  const runs = toRuns(tokens);
  let errorCount = 0;
  for (let i = 0; i < runs.length; i++) {
    const run = runs[i];
    if (run.status === "correct") continue;
    const next = runs[i + 1];
    const isSubstitutionPair =
      next && ((run.status === "missing" && next.status === "extra") || (run.status === "extra" && next.status === "missing"));
    if (isSubstitutionPair) {
      errorCount += Math.max(run.length, next.length);
      i += 1;
    } else {
      errorCount += run.length;
    }
  }
  return Math.max(0, 1 - errorCount / referenceCount);
}
