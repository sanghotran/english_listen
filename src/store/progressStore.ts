import { create } from "zustand";
import { computeAccuracy, diffWords } from "../utils/diff";
import type { Lesson } from "../types/lesson";
import type { Attempt, LevelProgress } from "../types/progress";

interface ProgressStore {
  attemptsByLesson: Record<string, Attempt[]>;
  submitAttempt: (lessonId: string, input: string, reference: string) => Promise<Attempt>;
  getAttemptsForLesson: (lessonId: string) => Attempt[];
  getLevelProgress: (lessons: Lesson[]) => LevelProgress[];
}

/**
 * Phase 1: attempts live only in memory (lost on reload). Phase 2/3 will persist
 * them to SQLite via services/tauri.ts and load them on startup instead — the
 * submitAttempt()/getLevelProgress() call sites in Practice.tsx/Progress.tsx
 * won't need to change since submitAttempt is already async.
 */
export const useProgressStore = create<ProgressStore>((set, get) => ({
  attemptsByLesson: {},

  submitAttempt: async (lessonId, input, reference) => {
    const tokens = diffWords(input, reference);
    const accuracy = computeAccuracy(tokens);
    const attempt: Attempt = {
      id: `${lessonId}-${Date.now()}`,
      lessonId,
      accuracy,
      attemptedAt: new Date().toISOString(),
    };
    set((state) => ({
      attemptsByLesson: {
        ...state.attemptsByLesson,
        [lessonId]: [...(state.attemptsByLesson[lessonId] ?? []), attempt],
      },
    }));
    return attempt;
  },

  getAttemptsForLesson: (lessonId) => get().attemptsByLesson[lessonId] ?? [],

  getLevelProgress: (lessons) => {
    const { attemptsByLesson } = get();
    const lessonById = new Map(lessons.map((l) => [l.id, l]));
    const byLevel = new Map<string, { lessonsCompleted: Set<string>; accuracySum: number; attemptCount: number }>();

    for (const [lessonId, attempts] of Object.entries(attemptsByLesson)) {
      const lesson = lessonById.get(lessonId);
      if (!lesson || attempts.length === 0) continue;
      const entry = byLevel.get(lesson.level) ?? {
        lessonsCompleted: new Set<string>(),
        accuracySum: 0,
        attemptCount: 0,
      };
      entry.lessonsCompleted.add(lessonId);
      for (const attempt of attempts) {
        entry.accuracySum += attempt.accuracy;
        entry.attemptCount += 1;
      }
      byLevel.set(lesson.level, entry);
    }

    return Array.from(byLevel.entries()).map(([level, entry]) => ({
      level: level as LevelProgress["level"],
      lessonsCompleted: entry.lessonsCompleted.size,
      averageAccuracy: entry.attemptCount === 0 ? 0 : entry.accuracySum / entry.attemptCount,
    }));
  },
}));
