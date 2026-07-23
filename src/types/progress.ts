import type { CefrLevel } from "./lesson";

export interface Attempt {
  id: number;
  lessonId: string;
  accuracy: number;
  attemptedAt: string;
}

export interface LevelProgress {
  level: CefrLevel;
  lessonsCompleted: number;
  averageAccuracy: number;
}
