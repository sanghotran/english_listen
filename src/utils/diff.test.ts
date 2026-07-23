import { describe, expect, it } from "vitest";
import { computeAccuracy, diffWords } from "./diff";

describe("diffWords", () => {
  it("marks an exact match as fully correct", () => {
    const tokens = diffWords("hello world", "hello world");
    expect(tokens).toEqual([
      { text: "hello", status: "correct" },
      { text: "world", status: "correct" },
    ]);
    expect(computeAccuracy(tokens)).toBe(1);
  });

  it("ignores case differences", () => {
    const tokens = diffWords("Hello WORLD", "hello world");
    expect(tokens.every((t) => t.status === "correct")).toBe(true);
  });

  it("ignores surrounding punctuation differences", () => {
    const tokens = diffWords("hello, world.", "hello world");
    expect(tokens.every((t) => t.status === "correct")).toBe(true);
  });

  it("flags a missing word", () => {
    const tokens = diffWords("hello", "hello world");
    expect(tokens).toEqual([
      { text: "hello", status: "correct" },
      { text: "world", status: "missing" },
    ]);
    expect(computeAccuracy(tokens)).toBe(0.5);
  });

  it("flags an extra word", () => {
    const tokens = diffWords("hello there world", "hello world");
    expect(tokens).toEqual([
      { text: "hello", status: "correct" },
      { text: "there", status: "extra" },
      { text: "world", status: "correct" },
    ]);
    // Standard WER counts insertions against accuracy too: 1 error / 2 reference words.
    expect(computeAccuracy(tokens)).toBe(0.5);
  });

  it("flags a substituted word as missing+extra", () => {
    const tokens = diffWords("hello planet", "hello world");
    expect(tokens).toEqual([
      { text: "hello", status: "correct" },
      { text: "world", status: "missing" },
      { text: "planet", status: "extra" },
    ]);
    expect(computeAccuracy(tokens)).toBe(0.5);
  });

  it("returns 0 accuracy for empty input against a non-empty reference", () => {
    const tokens = diffWords("", "hello world");
    expect(computeAccuracy(tokens)).toBe(0);
  });

  it("counts a multi-word substitution run as max(missing, extra), not their sum", () => {
    const tokens = diffWords("hello foo bar today", "hello world today");
    expect(tokens).toEqual([
      { text: "hello", status: "correct" },
      { text: "world", status: "missing" },
      { text: "foo", status: "extra" },
      { text: "bar", status: "extra" },
      { text: "today", status: "correct" },
    ]);
    // reference length 3 (hello, world, today); 1 substitution-run error of size max(1,2)=2 -> 1/3 wrong
    expect(computeAccuracy(tokens)).toBeCloseTo(1 / 3, 5);
  });

  it("returns 0 accuracy for an empty reference", () => {
    const tokens = diffWords("hello", "");
    expect(computeAccuracy(tokens)).toBe(0);
  });
});
