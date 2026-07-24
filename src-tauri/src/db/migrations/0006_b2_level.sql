-- Widen the level CHECK to allow 'B2' for the new TED source (ted.com talks — full-speed native
-- speech, well above the A1-B1 range dailydictation.com's categories cover). SQLite can't ALTER
-- a CHECK constraint in place, so the table is recreated and data copied across.
--
-- `legacy_alter_table` matters here: `segments`/`attempts` both have a foreign key referencing
-- `lessons`. Without it, SQLite's RENAME silently rewrites *their* stored schema to reference
-- "lessons_old" instead (its post-3.25 default behavior), which then dangles once lessons_old is
-- dropped below — "no such table: main.lessons_old" on the next segment/attempt insert. With it
-- on, the rename doesn't touch other tables' schemas, so they keep referencing plain "lessons",
-- which resolves correctly again once the new table below is created under that same name.
PRAGMA legacy_alter_table = ON;

ALTER TABLE lessons RENAME TO lessons_old;

CREATE TABLE lessons (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    level TEXT NOT NULL CHECK (level IN ('A1', 'A2', 'B1', 'B2')),
    category TEXT NOT NULL,
    audio_url TEXT NOT NULL,
    local_audio_path TEXT,
    page_url TEXT NOT NULL,
    published_at TEXT NOT NULL
);

INSERT INTO lessons SELECT * FROM lessons_old;
DROP TABLE lessons_old;

PRAGMA legacy_alter_table = OFF;
