import type { CefrLevel } from "../types/lesson";

/** Mirrors `chunk_size_for_level` in src-tauri/src/segments.rs — must stay in sync, since
 * `record_attempt` scores against the server's own segmentation of the same transcript. */
const CHUNK_SIZE: Record<CefrLevel, number> = { A1: 2, A2: 3, B1: 4 };

/** Mirrors `split_sentences` in src-tauri/src/segments.rs char-for-char so both sides agree
 * on segment boundaries for the same transcript. */
export function splitSentences(text: string): string[] {
  const sentences: string[] = [];
  let current = "";
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    current += c;
    if (c === "." || c === "!" || c === "?") {
      while (i + 1 < text.length && /[.!?]/.test(text[i + 1])) {
        i += 1;
        current += text[i];
      }
      const trimmed = current.trim();
      if (trimmed) sentences.push(trimmed);
      current = "";
    }
  }
  const trimmed = current.trim();
  if (trimmed) sentences.push(trimmed);
  return sentences;
}

/** Groups `transcript`'s sentences into level-sized chunks so a lesson is dictated a few
 * sentences at a time instead of all at once. Mirrors `segment_transcript` in segments.rs. */
export function segmentTranscript(transcript: string, level: CefrLevel): string[] {
  const sentences = splitSentences(transcript);
  const size = CHUNK_SIZE[level];
  const segments: string[] = [];
  for (let i = 0; i < sentences.length; i += size) {
    segments.push(sentences.slice(i, i + size).join(" "));
  }
  return segments;
}
