import { describe, expect, it } from "vitest";
import { segmentTranscript, splitSentences } from "./segments";

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
