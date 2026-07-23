export type CefrLevel = "A1" | "A2" | "B1";

export interface Lesson {
  id: string;
  title: string;
  level: CefrLevel;
  audioUrl: string;
  localAudioPath?: string;
  transcript: string;
  publishedAt: string;
}
