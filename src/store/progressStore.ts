import { create } from "zustand";
import * as tauriService from "../services/tauri";
import type { Attempt, LevelProgress } from "../types/progress";

interface ProgressStore {
  attemptsByLesson: Record<string, Attempt[]>;
  levelProgress: LevelProgress[];
  lessonCompletionById: Record<string, number>;
  loadAttempts: (lessonId: string) => Promise<void>;
  loadLevelProgress: () => Promise<void>;
  loadLessonProgress: () => Promise<void>;
  submitAttempt: (lessonId: string, segmentIndex: number, userTranscript: string) => Promise<Attempt>;
  getAttemptsForLesson: (lessonId: string) => Attempt[];
  getAttemptsForSegment: (lessonId: string, segmentIndex: number) => Attempt[];
}

export const useProgressStore = create<ProgressStore>((set, get) => ({
  attemptsByLesson: {},
  levelProgress: [],
  lessonCompletionById: {},

  loadAttempts: async (lessonId) => {
    const attempts = await tauriService.listAttempts(lessonId);
    set((state) => ({
      attemptsByLesson: { ...state.attemptsByLesson, [lessonId]: attempts },
    }));
  },

  loadLevelProgress: async () => {
    const levelProgress = await tauriService.getLevelProgress();
    set({ levelProgress });
  },

  loadLessonProgress: async () => {
    const rows = await tauriService.getLessonProgress();
    const lessonCompletionById: Record<string, number> = {};
    for (const row of rows) lessonCompletionById[row.lessonId] = row.completion;
    set({ lessonCompletionById });
  },

  // Accuracy is computed server-side by record_attempt (not trusted from the client) — see
  // src-tauri/src/commands/content.rs. The returned Attempt is the source of truth.
  submitAttempt: async (lessonId, segmentIndex, userTranscript) => {
    const attempt = await tauriService.recordAttempt(lessonId, segmentIndex, userTranscript);
    set((state) => ({
      attemptsByLesson: {
        ...state.attemptsByLesson,
        [lessonId]: [attempt, ...(state.attemptsByLesson[lessonId] ?? [])],
      },
    }));
    get().loadLevelProgress().catch(() => {});
    get().loadLessonProgress().catch(() => {});
    return attempt;
  },

  getAttemptsForLesson: (lessonId) => get().attemptsByLesson[lessonId] ?? [],

  getAttemptsForSegment: (lessonId, segmentIndex) =>
    (get().attemptsByLesson[lessonId] ?? []).filter((a) => a.segmentIndex === segmentIndex),
}));
