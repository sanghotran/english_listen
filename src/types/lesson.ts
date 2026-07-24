export type CefrLevel = "A1" | "A2" | "B1";

export interface Lesson {
  id: string;
  title: string;
  level: CefrLevel;
  category: string;
  audioUrl: string;
  localAudioPath?: string;
  pageUrl: string;
  publishedAt: string;
}
