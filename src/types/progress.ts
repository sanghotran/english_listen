export interface Attempt {
  lessonId: string;
  accuracy: number;
  attemptedAt: string;
}

export interface LevelProgress {
  level: "A1" | "A2" | "B1";
  lessonsCompleted: number;
  averageAccuracy: number;
}
