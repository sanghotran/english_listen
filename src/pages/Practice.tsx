import { useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import AudioPlayer from "../components/AudioPlayer/AudioPlayer";
import DictationInput from "../components/DictationInput/DictationInput";
import DiffViewer from "../components/DiffViewer/DiffViewer";
import ProgressTracker from "../components/ProgressTracker/ProgressTracker";
import { useDictationSession } from "../hooks/useDictationSession";
import { useLessonStore } from "../store/lessonStore";
import { useProgressStore } from "../store/progressStore";
import { errorMessage } from "../utils/error";
import { segmentTranscript } from "../utils/segments";
import "./Practice.css";

export default function Practice() {
  const { lessonId: rawLessonId } = useParams<{ lessonId: string }>();
  // Lesson ids are VOA article URLs (see fetch_new_lessons), so Home encodes them for the
  // route and we decode back here — the raw URL contains slashes that break route matching.
  const lessonId = rawLessonId ? decodeURIComponent(rawLessonId) : undefined;
  const { lessons, loadLessons, getLessonById, ensureAudioDownloaded } = useLessonStore();
  const { submitAttempt, getAttemptsForLesson, getAttemptsForSegment, loadAttempts } = useProgressStore();
  const [audioSrc, setAudioSrc] = useState<string | null>(null);
  const [audioError, setAudioError] = useState<string | null>(null);
  const [segmentIndex, setSegmentIndex] = useState(0);

  useEffect(() => {
    loadLessons();
  }, [loadLessons]);

  useEffect(() => {
    if (lessonId) loadAttempts(lessonId);
  }, [lessonId, loadAttempts]);

  const lesson = lessonId ? getLessonById(lessonId) : undefined;

  // A full ~5 minute transcript is too much to dictate in one pass, so it's split into a few
  // sentences at a time (more per segment at higher levels — see utils/segments.ts).
  const segments = useMemo(() => (lesson ? segmentTranscript(lesson.transcript, lesson.level) : []), [lesson]);

  useEffect(() => {
    setSegmentIndex(0);
  }, [lesson?.id]);

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
        console.error("audio download failed", err);
        if (!cancelled) setAudioError(errorMessage(err));
      });
    return () => {
      cancelled = true;
    };
  }, [lesson, ensureAudioDownloaded]);

  const { input, setInput, result, submit, reset } = useDictationSession(segments[segmentIndex] ?? "");

  useEffect(() => {
    reset();
  }, [lesson?.id, segmentIndex, reset]);

  if (lessons.length === 0) return <p>Loading lesson...</p>;
  if (!lesson) return <p role="alert">Lesson not found. <Link to="/">Back to lessons</Link></p>;

  const lessonAttempts = getAttemptsForLesson(lesson.id);
  const segmentAttempts = getAttemptsForSegment(lesson.id, segmentIndex);
  const bestAccuracy = segmentAttempts.length > 0 ? Math.max(...segmentAttempts.map((a) => a.accuracy)) : null;
  const attemptedSegments = new Set(lessonAttempts.map((a) => a.segmentIndex));

  const handleSubmit = () => {
    submit();
    submitAttempt(lesson.id, segmentIndex, input);
  };

  return (
    <div className="practice-page">
      <Link to="/" className="practice-page__back">
        ← Back
      </Link>
      <h1>{lesson.title}</h1>
      <ProgressTracker level={lesson.level} attemptCount={segmentAttempts.length} bestAccuracy={bestAccuracy} />
      {audioError && <p role="alert">Audio unavailable: {audioError}</p>}
      {audioSrc ? <AudioPlayer src={audioSrc} /> : !audioError && <p className="practice-page__status">Downloading audio…</p>}

      <div className="segment-nav">
        <button
          type="button"
          className="btn"
          onClick={() => setSegmentIndex((i) => i - 1)}
          disabled={segmentIndex === 0}
        >
          ← Prev
        </button>
        <div className="segment-nav__dots">
          {segments.map((_, i) => (
            <button
              key={i}
              type="button"
              className={`segment-nav__dot ${i === segmentIndex ? "segment-nav__dot--active" : ""} ${
                attemptedSegments.has(i) ? "segment-nav__dot--done" : ""
              }`}
              onClick={() => setSegmentIndex(i)}
              aria-label={`Segment ${i + 1} of ${segments.length}`}
              aria-current={i === segmentIndex}
            />
          ))}
        </div>
        <button
          type="button"
          className="btn"
          onClick={() => setSegmentIndex((i) => i + 1)}
          disabled={segmentIndex >= segments.length - 1}
        >
          Next →
        </button>
      </div>
      <p className="segment-nav__label">
        Segment {segmentIndex + 1} / {segments.length}
      </p>

      <DictationInput value={input} onChange={setInput} onSubmit={handleSubmit} />
      {result && <DiffViewer tokens={result.tokens} accuracy={result.accuracy} />}
    </div>
  );
}
