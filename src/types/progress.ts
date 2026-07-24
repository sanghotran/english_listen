import type { CefrLevel } from "./lesson";

export interface Attempt {
  id: number;
  lessonId: string;
  segmentIndex: number;
  accuracy: number;
  attemptedAt: string;
}

export interface LevelProgress {
  level: CefrLevel;
  lessonsCompleted: number;
  averageAccuracy: number;
}

export interface LessonProgress {
  lessonId: string;
  /** Fraction (0.0-1.0) of the lesson's segments with at least one recorded attempt. */
  completion: number;
}
