import { create } from "zustand";
import * as tauriService from "../services/tauri";
import type { CefrLevel, Lesson } from "../types/lesson";
import { errorMessage } from "../utils/error";

interface LessonStore {
  lessons: Lesson[];
  levelFilter: CefrLevel | "ALL";
  isLoading: boolean;
  error: string | null;
  setLevelFilter: (level: CefrLevel | "ALL") => void;
  loadLessons: () => Promise<void>;
  refreshFromVoa: () => Promise<number>;
  ensureAudioDownloaded: (lessonId: string) => Promise<string>;
  getLessonById: (id: string) => Lesson | undefined;
  getLessonsByLevel: (level: CefrLevel | "ALL") => Lesson[];
}

export const useLessonStore = create<LessonStore>((set, get) => ({
  lessons: [],
  levelFilter: "ALL",
  isLoading: false,
  error: null,

  setLevelFilter: (level) => set({ levelFilter: level }),

  loadLessons: async () => {
    if (get().lessons.length > 0) return;
    set({ isLoading: true, error: null });
    try {
      const lessons = await tauriService.listLessons();
      set({ lessons, isLoading: false });
    } catch (err) {
      console.error("loadLessons failed", err);
      set({ error: errorMessage(err), isLoading: false });
    }
  },

  refreshFromVoa: async () => {
    set({ isLoading: true, error: null });
    try {
      const result = await tauriService.fetchNewLessons();
      const lessons = await tauriService.listLessons();
      set({ lessons, isLoading: false });
      return result.new;
    } catch (err) {
      console.error("refreshFromVoa failed", err);
      set({ error: errorMessage(err), isLoading: false });
      throw err;
    }
  },

  ensureAudioDownloaded: (lessonId) => tauriService.ensureAudioUrl(lessonId),

  getLessonById: (id) => get().lessons.find((l) => l.id === id),

  getLessonsByLevel: (level) => {
    const { lessons } = get();
    return level === "ALL" ? lessons : lessons.filter((l) => l.level === level);
  },
}));
