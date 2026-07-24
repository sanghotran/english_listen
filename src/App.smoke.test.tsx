/** @vitest-environment jsdom */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { useLessonStore } from "./store/lessonStore";
import { useProgressStore } from "./store/progressStore";

const LESSON = {
  id: "lesson-test",
  title: "Test Lesson",
  level: "A2" as const,
  category: "english-conversations",
  audioUrl: "https://example.com/audio.mp3",
  localAudioPath: "/tmp/lesson-test.mp3",
  pageUrl: "https://example.com/page",
  publishedAt: "2026-01-01",
};

const SEGMENTS = [
  { id: 1, lessonId: LESSON.id, position: 0, content: "hello world today", timeStart: 0, timeEnd: 3 },
];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
    switch (cmd) {
      case "list_lessons":
        return [LESSON];
      case "list_segments":
        return SEGMENTS;
      case "get_lesson_audio_path":
        return LESSON.localAudioPath;
      case "download_audio":
        return null;
      case "record_attempt":
        return {
          id: 1,
          lessonId: args?.lessonId,
          segmentIndex: args?.segmentIndex,
          accuracy: 0.67,
          attemptedAt: "2026-01-01T00:00:00Z",
        };
      case "list_attempts":
        return [];
      case "get_level_progress":
        return [];
      case "get_lesson_progress":
        return [];
      default:
        throw new Error(`unmocked tauri command: ${cmd}`);
    }
  }),
  convertFileSrc: (path: string) => `asset://${path}`,
}));

beforeEach(() => {
  useLessonStore.setState({ lessons: [], levelFilter: "ALL", isLoading: false, error: null, segmentsByLesson: {} });
  useProgressStore.setState({ attemptsByLesson: {}, levelProgress: [], lessonCompletionById: {} });
  window.history.pushState({}, "", "/");
});

describe("dictation flow (smoke test, no real browser available in this sandbox)", () => {
  it("lets a user browse to a lesson, type a dictation attempt, and see the diff + accuracy", async () => {
    const user = userEvent.setup();
    render(<App />);

    // Home: lesson list loads from the (mocked) Tauri backend.
    expect(await screen.findByText(/Test Lesson/)).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: /Test Lesson/ }));

    // Practice page for that lesson.
    expect(await screen.findByRole("heading", { name: "Test Lesson" })).toBeInTheDocument();

    const textarea = await screen.findByPlaceholderText(/Type what you hear/);
    await user.type(textarea, "hello word today");
    await user.click(screen.getByRole("button", { name: "Check" }));

    await waitFor(() => expect(screen.getByText(/Accuracy:/)).toBeInTheDocument());
    // "word" vs "world" is a substitution -> 1 error / 3 reference words = 67% (computed
    // client-side by useDictationSession for immediate feedback; record_attempt's server-side
    // score is mocked separately above and drives the persisted Attempt, not this token view).
    expect(screen.getByText("Accuracy: 67%")).toBeInTheDocument();
    expect(screen.getByText("hello")).toBeInTheDocument();
    expect(screen.getByText(/world/)).toBeInTheDocument();
  });
});
