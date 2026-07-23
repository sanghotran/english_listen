// Thin wrappers around @tauri-apps/api invoke() for Rust commands
// e.g. fetchNewLessons(), downloadAudio(lessonId), getLessonAudioPath(lessonId)
import { invoke } from "@tauri-apps/api/core";

export function fetchNewLessons() {
  return invoke("fetch_new_lessons");
}
