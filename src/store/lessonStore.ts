import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import * as tauriService from "../services/tauri";
import type { CefrLevel, Lesson, RefreshProgress } from "../types/lesson";
import type { Segment } from "../types/segment";
import { errorMessage } from "../utils/error";

interface LessonStore {
  lessons: Lesson[];
  levelFilter: CefrLevel | "ALL";
  isLoading: boolean;
  error: string | null;
  refreshProgress: RefreshProgress | null;
  segmentsByLesson: Record<string, Segment[]>;
  setLevelFilter: (level: CefrLevel | "ALL") => void;
  loadLessons: () => Promise<void>;
  refreshLessons: () => Promise<number>;
  ensureAudioDownloaded: (lessonId: string) => Promise<string>;
  getLessonById: (id: string) => Lesson | undefined;
  getLessonsByLevel: (level: CefrLevel | "ALL") => Lesson[];
  loadSegments: (lessonId: string) => Promise<void>;
  getSegmentsForLesson: (lessonId: string) => Segment[];
}

export const useLessonStore = create<LessonStore>((set, get) => ({
  lessons: [],
  levelFilter: "ALL",
  isLoading: false,
  error: null,
  refreshProgress: null,
  segmentsByLesson: {},

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

  refreshLessons: async () => {
    set({ isLoading: true, error: null, refreshProgress: null });
    const unlisten = await listen<RefreshProgress>("lessons-refresh-progress", (event) => {
      set({ refreshProgress: event.payload });
    });
    try {
      const result = await tauriService.fetchNewLessons();
      const lessons = await tauriService.listLessons();
      set({ lessons, isLoading: false, refreshProgress: null });
      return result.new;
    } catch (err) {
      console.error("refreshLessons failed", err);
      set({ error: errorMessage(err), isLoading: false, refreshProgress: null });
      throw err;
    } finally {
      unlisten();
    }
  },

  ensureAudioDownloaded: (lessonId) => tauriService.ensureAudioUrl(lessonId),

  getLessonById: (id) => get().lessons.find((l) => l.id === id),

  getLessonsByLevel: (level) => {
    const { lessons } = get();
    return level === "ALL" ? lessons : lessons.filter((l) => l.level === level);
  },

  loadSegments: async (lessonId) => {
    const segments = await tauriService.listSegments(lessonId);
    set((state) => ({
      segmentsByLesson: { ...state.segmentsByLesson, [lessonId]: segments },
    }));
  },

  getSegmentsForLesson: (lessonId) => get().segmentsByLesson[lessonId] ?? [],
}));
