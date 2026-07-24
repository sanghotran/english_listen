-- Source switched from VOA (whole-lesson transcript, no timestamps) to dailydictation.com
-- (per-sentence "challenges" with exact timeStart/timeEnd). Segments are now authored data,
-- not computed client-side, so the schema shape changes rather than just growing a column;
-- old VOA-scraped rows are no longer fetchable from their source and are dropped.
DROP TABLE IF EXISTS attempts;
DROP TABLE IF EXISTS lessons;

CREATE TABLE lessons (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    level TEXT NOT NULL CHECK (level IN ('A1', 'A2', 'B1')),
    category TEXT NOT NULL,
    audio_url TEXT NOT NULL,
    local_audio_path TEXT,
    page_url TEXT NOT NULL,
    published_at TEXT NOT NULL
);

CREATE TABLE segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lesson_id TEXT NOT NULL REFERENCES lessons(id),
    position INTEGER NOT NULL,
    content TEXT NOT NULL,
    time_start REAL NOT NULL,
    time_end REAL NOT NULL,
    UNIQUE (lesson_id, position)
);
CREATE INDEX idx_segments_lesson_id ON segments(lesson_id);

CREATE TABLE attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lesson_id TEXT NOT NULL REFERENCES lessons(id),
    segment_index INTEGER NOT NULL,
    accuracy REAL NOT NULL,
    attempted_at TEXT NOT NULL,
    user_transcript TEXT NOT NULL DEFAULT '',
    correct_count INTEGER NOT NULL DEFAULT 0,
    missing_count INTEGER NOT NULL DEFAULT 0,
    extra_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_attempts_lesson_id ON attempts(lesson_id);
