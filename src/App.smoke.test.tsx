/** @vitest-environment jsdom */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { useLessonStore } from "./store/lessonStore";
import { useProgressStore } from "./store/progressStore";

const LESSONS = [
  {
    id: "lesson-test",
    title: "Test Lesson",
    level: "A2" as const,
    sourceShow: "Test Show",
    audioUrl: "https://example.com/audio.mp3",
    transcript: "hello world today",
    pageUrl: "https://example.com/page",
    publishedAt: "2026-01-01",
  },
];

beforeEach(() => {
  useLessonStore.setState({ lessons: [], isLoading: false, error: null });
  useProgressStore.setState({ attemptsByLesson: {} });
  window.history.pushState({}, "", "/");
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response(JSON.stringify(LESSONS), { status: 200 })),
  );
});

describe("dictation flow (smoke test, no real browser available in this sandbox)", () => {
  it("lets a user browse to a lesson, type a dictation attempt, and see the diff + accuracy", async () => {
    const user = userEvent.setup();
    render(<App />);

    // Home: lesson list loads from the (mocked) fixture fetch.
    expect(await screen.findByText(/Test Lesson/)).toBeInTheDocument();

    await user.click(screen.getByRole("link", { name: /Test Lesson/ }));

    // Practice page for that lesson.
    expect(await screen.findByRole("heading", { name: "Test Lesson" })).toBeInTheDocument();

    const textarea = screen.getByPlaceholderText(/Type what you hear/);
    await user.type(textarea, "hello word today");
    await user.click(screen.getByRole("button", { name: "Check" }));

    await waitFor(() => expect(screen.getByText(/Accuracy:/)).toBeInTheDocument());
    // "word" vs "world" is a substitution -> 1 error / 3 reference words = 67%.
    expect(screen.getByText("Accuracy: 67%")).toBeInTheDocument();
    expect(screen.getByText("hello")).toBeInTheDocument();
    expect(screen.getByText(/world/)).toBeInTheDocument();
  });
});
