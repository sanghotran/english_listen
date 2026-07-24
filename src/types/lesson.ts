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

/** Emitted by the Rust `fetch_new_lessons` command (event `lessons-refresh-progress`) while a
 * refresh is in flight — a full first run walks 1000+ pages at a paced rate and can take
 * several minutes, so the frontend needs live progress rather than one final result. */
export interface RefreshProgress {
  processed: number;
  total: number;
  newCount: number;
  category: string;
}
