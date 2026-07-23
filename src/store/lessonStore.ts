import { create } from "zustand";
import type { CefrLevel, Lesson } from "../types/lesson";

interface LessonStore {
  lessons: Lesson[];
  isLoading: boolean;
  error: string | null;
  loadLessons: () => Promise<void>;
  getLessonById: (id: string) => Lesson | undefined;
  getLessonsByLevel: (level: CefrLevel | "ALL") => Lesson[];
}

/**
 * Phase 1: lessons come from a static fixture file (public/fixtures/lessons.json)
 * seeded with real VOA Learning English episodes. Phase 4+ will swap loadLessons()
 * to call services/tauri.ts instead — the public interface stays the same so
 * Home.tsx / Practice.tsx won't need to change.
 */
export const useLessonStore = create<LessonStore>((set, get) => ({
  lessons: [],
  isLoading: false,
  error: null,

  loadLessons: async () => {
    if (get().lessons.length > 0) return;
    set({ isLoading: true, error: null });
    try {
      const res = await fetch("/fixtures/lessons.json");
      if (!res.ok) throw new Error(`Failed to load lessons: ${res.status}`);
      const lessons: Lesson[] = await res.json();
      set({ lessons, isLoading: false });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : String(err), isLoading: false });
    }
  },

  getLessonById: (id) => get().lessons.find((l) => l.id === id),

  getLessonsByLevel: (level) => {
    const { lessons } = get();
    return level === "ALL" ? lessons : lessons.filter((l) => l.level === level);
  },
}));
