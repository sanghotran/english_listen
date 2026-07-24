import { describe, expect, it } from "vitest";
import { segmentAudioBounds, segmentTranscript, splitSentences } from "./segments";

describe("splitSentences", () => {
  it("splits on sentence terminators", () => {
    expect(splitSentences("One. Two! Three? Four.")).toEqual(["One.", "Two!", "Three?", "Four."]);
  });

  it("keeps multi-punctuation together", () => {
    expect(splitSentences("Wait... really?! Yes.")).toEqual(["Wait...", "really?!", "Yes."]);
  });

  it("treats trailing text without a terminator as its own sentence", () => {
    expect(splitSentences("One. trailing fragment")).toEqual(["One.", "trailing fragment"]);
  });

  it("returns no sentences for empty input", () => {
    expect(splitSentences("")).toEqual([]);
  });
});

describe("segmentTranscript", () => {
  const transcript = "One. Two. Three. Four. Five. Six. Seven. Eight.";

  it("groups by level chunk size", () => {
    expect(segmentTranscript(transcript, "A1")).toEqual(["One. Two.", "Three. Four.", "Five. Six.", "Seven. Eight."]);
    expect(segmentTranscript(transcript, "A2")).toEqual(["One. Two. Three.", "Four. Five. Six.", "Seven. Eight."]);
    expect(segmentTranscript(transcript, "B1")).toEqual(["One. Two. Three. Four.", "Five. Six. Seven. Eight."]);
  });

  it("returns no segments for empty input", () => {
    expect(segmentTranscript("", "B1")).toEqual([]);
  });
});

describe("segmentAudioBounds", () => {
  it("splits proportionally by word count across equal-length segments", () => {
    const segments = ["one two", "three four", "five six"];
    expect(segmentAudioBounds(segments, 0)).toEqual({ startFraction: 0, endFraction: 1 / 3 });
    expect(segmentAudioBounds(segments, 1)).toEqual({ startFraction: 1 / 3, endFraction: 2 / 3 });
    expect(segmentAudioBounds(segments, 2)).toEqual({ startFraction: 2 / 3, endFraction: 1 });
  });

  it("weighs longer segments with a proportionally larger fraction", () => {
    const segments = ["one two three", "four"];
    expect(segmentAudioBounds(segments, 0)).toEqual({ startFraction: 0, endFraction: 0.75 });
    expect(segmentAudioBounds(segments, 1)).toEqual({ startFraction: 0.75, endFraction: 1 });
  });

  it("does not divide by zero for an empty segment list", () => {
    expect(segmentAudioBounds([], 0)).toEqual({ startFraction: 0, endFraction: 0 });
  });
});
