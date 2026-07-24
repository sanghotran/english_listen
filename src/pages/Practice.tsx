import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import AudioPlayer from "../components/AudioPlayer/AudioPlayer";
import DictationInput from "../components/DictationInput/DictationInput";
import DiffViewer from "../components/DiffViewer/DiffViewer";
import { MascotBunny } from "../components/Mascot/Mascot";
import ProgressTracker from "../components/ProgressTracker/ProgressTracker";
import { useDictationSession } from "../hooks/useDictationSession";
import { useLessonStore } from "../store/lessonStore";
import { useProgressStore } from "../store/progressStore";
import { errorMessage } from "../utils/error";
import "./Practice.css";

export default function Practice() {
  const { lessonId } = useParams<{ lessonId: string }>();
  const { lessons, loadLessons, getLessonById, ensureAudioDownloaded, loadSegments, getSegmentsForLesson } =
    useLessonStore();
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

  useEffect(() => {
    if (lessonId) loadSegments(lessonId);
  }, [lessonId, loadSegments]);

  const lesson = lessonId ? getLessonById(lessonId) : undefined;
  const segments = lessonId ? getSegmentsForLesson(lessonId) : [];

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

  const segment = segments[segmentIndex];
  const { input, setInput, result, submit, reset } = useDictationSession(segment?.content ?? "");

  useEffect(() => {
    reset();
  }, [lesson?.id, segmentIndex, reset]);

  if (lessons.length === 0) return <p>Loading lesson...</p>;
  if (!lesson) return <p role="alert">Lesson not found. <Link to="/">Back to lessons</Link></p>;
  if (segments.length === 0) return <p>Loading segments...</p>;

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
      <div className="practice-page__title">
        <MascotBunny size={36} />
        <h1>{lesson.title}</h1>
      </div>
      <ProgressTracker level={lesson.level} attemptCount={segmentAttempts.length} bestAccuracy={bestAccuracy} />
      {audioError && <p role="alert">Audio unavailable: {audioError}</p>}
      {audioSrc && segment ? (
        <AudioPlayer src={audioSrc} bounds={{ startTime: segment.timeStart, endTime: segment.timeEnd }} />
      ) : (
        !audioError && <p className="practice-page__status">Downloading audio…</p>
      )}

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
