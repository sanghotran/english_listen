import { create } from "zustand";
import * as tauriService from "../services/tauri";
import type { Attempt, LevelProgress } from "../types/progress";

interface ProgressStore {
  attemptsByLesson: Record<string, Attempt[]>;
  levelProgress: LevelProgress[];
  loadAttempts: (lessonId: string) => Promise<void>;
  loadLevelProgress: () => Promise<void>;
  submitAttempt: (lessonId: string, userTranscript: string) => Promise<Attempt>;
  getAttemptsForLesson: (lessonId: string) => Attempt[];
}

export const useProgressStore = create<ProgressStore>((set, get) => ({
  attemptsByLesson: {},
  levelProgress: [],

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

  // Accuracy is computed server-side by record_attempt (not trusted from the client) — see
  // src-tauri/src/commands/content.rs. The returned Attempt is the source of truth.
  submitAttempt: async (lessonId, userTranscript) => {
    const attempt = await tauriService.recordAttempt(lessonId, userTranscript);
    set((state) => ({
      attemptsByLesson: {
        ...state.attemptsByLesson,
        [lessonId]: [attempt, ...(state.attemptsByLesson[lessonId] ?? [])],
      },
    }));
    get().loadLevelProgress().catch(() => {});
    return attempt;
  },

  getAttemptsForLesson: (lessonId) => get().attemptsByLesson[lessonId] ?? [],
}));
