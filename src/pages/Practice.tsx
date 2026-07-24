import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import AudioPlayer from "../components/AudioPlayer/AudioPlayer";
import DictationInput from "../components/DictationInput/DictationInput";
import DiffViewer from "../components/DiffViewer/DiffViewer";
import ProgressTracker from "../components/ProgressTracker/ProgressTracker";
import { useDictationSession } from "../hooks/useDictationSession";
import { useLessonStore } from "../store/lessonStore";
import { useProgressStore } from "../store/progressStore";
import "./Practice.css";

export default function Practice() {
  const { lessonId: rawLessonId } = useParams<{ lessonId: string }>();
  // Lesson ids are VOA article URLs (see fetch_new_lessons), so Home encodes them for the
  // route and we decode back here — the raw URL contains slashes that break route matching.
  const lessonId = rawLessonId ? decodeURIComponent(rawLessonId) : undefined;
  const { lessons, loadLessons, getLessonById, ensureAudioDownloaded } = useLessonStore();
  const { submitAttempt, getAttemptsForLesson, loadAttempts } = useProgressStore();
  const [audioSrc, setAudioSrc] = useState<string | null>(null);
  const [audioError, setAudioError] = useState<string | null>(null);

  useEffect(() => {
    loadLessons();
  }, [loadLessons]);

  useEffect(() => {
    if (lessonId) loadAttempts(lessonId);
  }, [lessonId, loadAttempts]);

  const lesson = lessonId ? getLessonById(lessonId) : undefined;

  useEffect(() => {
    if (!lesson) return;
    let cancelled = false;
    setAudioSrc(null);
    setAudioError(null);
    ensureAudioDownloaded(lesson.id)
      .then((src) => {
        if (!cancelled) setAudioSrc(src);
      })
      .catch((err) => {
        if (!cancelled) setAudioError(err instanceof Error ? err.message : String(err));
      });
    return () => {
      cancelled = true;
    };
  }, [lesson, ensureAudioDownloaded]);

  const { input, setInput, result, submit } = useDictationSession(lesson?.transcript ?? "");

  if (lessons.length === 0) return <p>Loading lesson...</p>;
  if (!lesson) return <p role="alert">Lesson not found. <Link to="/">Back to lessons</Link></p>;

  const attempts = getAttemptsForLesson(lesson.id);
  const bestAccuracy = attempts.length > 0 ? Math.max(...attempts.map((a) => a.accuracy)) : null;

  const handleSubmit = () => {
    submit();
    submitAttempt(lesson.id, input);
  };

  return (
    <div className="practice-page">
      <Link to="/" className="practice-page__back">
        ← Back
      </Link>
      <h1>{lesson.title}</h1>
      <ProgressTracker level={lesson.level} attemptCount={attempts.length} bestAccuracy={bestAccuracy} />
      {audioError && <p role="alert">Audio unavailable: {audioError}</p>}
      {audioSrc ? <AudioPlayer src={audioSrc} /> : !audioError && <p className="practice-page__status">Downloading audio…</p>}
      <DictationInput value={input} onChange={setInput} onSubmit={handleSubmit} />
      {result && <DiffViewer tokens={result.tokens} accuracy={result.accuracy} />}
    </div>
  );
}
