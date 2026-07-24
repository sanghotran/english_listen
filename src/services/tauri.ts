// Thin, typed wrappers around @tauri-apps/api invoke() for the Rust commands
// (src-tauri/src/commands/{content,audio}.rs). JS camelCase arg names are auto-mapped
// to the commands' snake_case Rust parameters by Tauri's IPC layer.
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { Lesson } from "../types/lesson";
import type { Attempt, LessonProgress, LevelProgress } from "../types/progress";
import type { Segment } from "../types/segment";

export interface FetchResult {
  new: number;
}

export function listLessons(): Promise<Lesson[]> {
  return invoke("list_lessons");
}

export function getLesson(id: string): Promise<Lesson> {
  return invoke("get_lesson", { id });
}

export function listSegments(lessonId: string): Promise<Segment[]> {
  return invoke("list_segments", { lessonId });
}

export function fetchNewLessons(): Promise<FetchResult> {
  return invoke("fetch_new_lessons");
}

export async function downloadAudio(lessonId: string): Promise<void> {
  await invoke("download_audio", { lessonId });
}

export function getLessonAudioPath(lessonId: string): Promise<string | null> {
  return invoke("get_lesson_audio_path", { lessonId });
}

/** Resolves to a webview-loadable URL for the lesson's audio, downloading it first if needed. */
export async function ensureAudioUrl(lessonId: string): Promise<string> {
  let localPath = await getLessonAudioPath(lessonId);
  if (!localPath) {
    await downloadAudio(lessonId);
    localPath = await getLessonAudioPath(lessonId);
  }
  if (!localPath) throw new Error(`audio for lesson '${lessonId}' failed to download`);
  return convertFileSrc(localPath);
}

export function recordAttempt(lessonId: string, segmentIndex: number, userTranscript: string): Promise<Attempt> {
  return invoke("record_attempt", { lessonId, segmentIndex, userTranscript });
}

export function listAttempts(lessonId: string): Promise<Attempt[]> {
  return invoke("list_attempts", { lessonId });
}

export function getLevelProgress(): Promise<LevelProgress[]> {
  return invoke("get_level_progress");
}

export function getLessonProgress(): Promise<LessonProgress[]> {
  return invoke("get_lesson_progress");
}
