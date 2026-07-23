import { useEffect } from "react";
import { Link, useParams } from "react-router-dom";
import AudioPlayer from "../components/AudioPlayer/AudioPlayer";
import DictationInput from "../components/DictationInput/DictationInput";
import DiffViewer from "../components/DiffViewer/DiffViewer";
import ProgressTracker from "../components/ProgressTracker/ProgressTracker";
import { useDictationSession } from "../hooks/useDictationSession";
import { useLessonStore } from "../store/lessonStore";
import { useProgressStore } from "../store/progressStore";

export default function Practice() {
  const { lessonId } = useParams<{ lessonId: string }>();
  const { lessons, loadLessons, getLessonById } = useLessonStore();
  const { submitAttempt, getAttemptsForLesson } = useProgressStore();

  useEffect(() => {
    loadLessons();
  }, [loadLessons]);

  const lesson = lessonId ? getLessonById(lessonId) : undefined;
  const { input, setInput, result, submit } = useDictationSession(lesson?.transcript ?? "");

  if (lessons.length === 0) return <p>Loading lesson...</p>;
  if (!lesson) return <p role="alert">Lesson not found. <Link to="/">Back to lessons</Link></p>;

  const attempts = getAttemptsForLesson(lesson.id);
  const bestAccuracy = attempts.length > 0 ? Math.max(...attempts.map((a) => a.accuracy)) : null;

  const handleSubmit = () => {
    submit();
    submitAttempt(lesson.id, input, lesson.transcript);
  };

  return (
    <div className="practice-page">
      <Link to="/">← Back</Link>
      <h1>{lesson.title}</h1>
      <ProgressTracker level={lesson.level} attemptCount={attempts.length} bestAccuracy={bestAccuracy} />
      <AudioPlayer src={lesson.audioUrl} />
      <DictationInput value={input} onChange={setInput} onSubmit={handleSubmit} />
      {result && <DiffViewer tokens={result.tokens} accuracy={result.accuracy} />}
    </div>
  );
}
