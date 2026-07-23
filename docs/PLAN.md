# Kế hoạch triển khai: English Listen (dictation app)

## Context

Repo hiện chỉ có bộ khung Tauri + React (toàn bộ file placeholder, chưa có logic thật). Mục tiêu là triển khai từng phần để có sản phẩm demo được sớm, tránh dồn hết rủi ro (scraping VOA thật) vào cuối. Đã **verify trực tiếp bằng curl** cấu trúc RSS/HTML thật của VOA Learning English (không đoán) — các URL/selector dưới đây là dữ liệu thật, không phải giả định.

### Sự thật đã verify về VOA (2026-07-23)
- VOA có bot-protection: request không có `User-Agent` giống browser sẽ bị `403`. Client Rust `reqwest` **phải** set UA kiểu Chrome.
- Không có 1 RSS feed tổng — có ~46 feed theo từng show tại `/rssfeeds`, ví dụ đã confirm hoạt động (HTTP 200):
  - `As It Is` → `/api/zkm-ql-vomx-tpej-rqi` (B1)
  - `Ask a Teacher` → `/api/zti_qvl-vomx-tpekgvqr` (A2)
  - `Everyday Grammar` → `/api/zoroqql-vomx-tpeptpqq` (A2)
  - `Words and Their Stories` → `/api/zmypyl-vomx-tpeyry_` (B1)
- RSS item **không** có mp3/transcript — chỉ có `title`, `link`, `guid`, `pubDate`. Phải mở từng episode page (`link`) để lấy:
  - Audio: `<audio src="https://voa-audio.voanews.eu/...mp3" ...>` (đã confirm thật, ví dụ episode "Sew and Knit")
  - Transcript: các `<p>` trong `div.wsw`, lấy đến trước `<h2>Words in This Story</h2>` (phần sau là glossary, bỏ qua).
- **Không phải item nào cũng có đủ audio+transcript** (có item chỉ là video, có item audio-only không transcript) → ingestion phải tự skip, không coi là lỗi.
- **Nội dung A1 chưa có nguồn RSS rõ ràng** ("Let's Learn English Level 1/2" feed chỉ trả về trang tĩnh, không phải episode) → để sau (Phase 8), không block các phase khác. Bắt đầu với A2/B1 từ 4 feed đã confirm.

## Các phase triển khai

### Phase 1 — Frontend dictation loop bằng fixture data (không cần Rust/SQLite/Tauri)
- Tạo `public/fixtures/lessons.json` + 2-3 bài mp3/transcript thật (lấy tay từ VOA, ví dụ "Sew and Knit").
- Viết thật `src/utils/diff.ts`, `hooks/useAudioPlayer.ts`, `hooks/useDictationSession.ts`, `components/AudioPlayer`, `DictationInput`, `DiffViewer`, `LevelSelector`, `pages/Home.tsx`, `pages/Practice.tsx`.
- `lessonStore.loadLessons()` tạm fetch fixture JSON (giữ interface giống Phase 6 để sau chỉ đổi implementation, không đổi chỗ gọi).
- `progressStore` tạm lưu attempt trong memory (chưa persist).
- **Done khi**: `npm run dev` (chưa cần Tauri) chạy được full luồng chọn bài → nghe → gõ → xem diff + % điểm, trong browser thường.

### Phase 2 — SQLite schema v2 + Rust `db` module
- Thêm migration `0002_refine.sql` (additive): `lessons` thêm `guid` (unique index), `source_show`, `word_count`; `attempts` thêm `user_transcript`, `correct_count`, `missing_count`, `extra_count` + index theo `lesson_id`.
- **Không** thêm bảng `level_progress` materialized — tính aggregate on-the-fly bằng `GROUP BY` (dữ liệu ít, tránh bug drift khi update quên đồng bộ).
- `db/mod.rs`: `init_pool()` dùng `app.path().app_data_dir()`, tạo file `english_listen.db`, chạy `sqlx::migrate!`. Tạo thêm dir `audio/` cùng chỗ để cache mp3.
- **Done khi**: `cargo test` chạy migration trên `sqlite::memory:`, insert/query lesson + attempt OK.

### Phase 3 — Tauri command layer (IPC contract, 8 commands)
`list_lessons`, `get_lesson`, `fetch_new_lessons`, `download_audio`, `get_lesson_audio_path`, `record_attempt`, `list_attempts`, `get_level_progress`.
- Thêm `src-tauri/src/error.rs` — enum `AppError` (Database/Network/NotFound/Parse), `#[serde(tag="kind", content="message")]` để frontend match theo `err.kind`.
- Accuracy tính lại ở server-side trong `record_attempt` (không tin số từ client).
- Audio phát qua Tauri asset protocol + `convertFileSrc()`, không mở URL VOA trực tiếp trong webview.
- **Done khi**: gọi được cả 8 command từ dev console với 1 row insert tay, `Practice.tsx` round-trip end-to-end với data thật (chưa cần scraping).

### Phase 4 — Ingestion VOA thật (dùng cấu trúc đã verify ở trên)
- Thêm crate `rss = "2"`, `scraper = "0.19"`.
- `scraper/feeds.rs`: config tĩnh 4 feed đã verify + level gán cứng theo show (không đoán per-item).
- `scraper/voa_rss.rs::parse_channel()`, `scraper/transcript.rs::extract_episode()` — trả `Option`, `None` nghĩa là skip (không phải lỗi).
- Upsert theo `guid` (`ON CONFLICT DO UPDATE`) → re-run `fetch_new_lessons` tự nhiên idempotent, không cần bảng "last seen" riêng.
- `download_audio`: stream file về `{app_data_dir}/audio/{lesson_id}.mp3`.
- **Done khi**: `fetch_new_lessons` lấy được lesson thật (A2/B1) có audio+transcript chạy được, chạy lại lần 2 báo `new: 0`.

### Phase 5 — Diff & scoring (làm song song 2-4 được)
- Dùng lib `diff` (jsdiff) thay vì tự viết Myers-diff.
- `normalize()`: lowercase + strip dấu câu ngoài từ, giữ `'`/`-` bên trong.
- `computeAccuracy()` = 1 − word-error-rate (chuẩn WER, giống ASR scoring).
- Giới hạn biết trước (ghi chú lại, không giấu): chưa xử lý contraction (`don't` vs `do not`) — VOA transcript có dùng contraction nên sẽ ảnh hưởng độ chính xác; để cải thiện ở Phase 8.
- **Done khi**: unit test (`vitest`) pass các case: khớp hệt, khác hoa/thường, khác dấu câu, thiếu từ, dư từ, sai từ, input rỗng.

### Phase 6 — Wiring state management
- `lessonStore`: `lessons`, `levelFilter`, `loadLessons()`, `refreshFromVoa()`, `ensureAudioDownloaded()` (trả URL đã `convertFileSrc`).
- `progressStore`: `attemptsByLesson`, `levelProgress`, `submitAttempt()` (gọi `diffWords` + `services/tauri.recordAttempt`).
- `Practice.tsx` orchestrate: lấy lesson → đảm bảo audio đã tải → `useAudioPlayer` + `useDictationSession` → submit → render `DiffViewer` + `ProgressTracker`.
- Gỡ fixture khỏi hot path, chuyển hẳn sang gọi Tauri thật.

### Phase 7 — Testing & CI
- `vitest`: diff/scoring + store logic (mock `services/tauri`).
- `cargo test`: parser RSS/HTML dùng **fixture đã lấy thật hôm nay** (feed XML của "As It Is", HTML episode "Sew and Knit") — không tự viết HTML giả.
- Test thủ công: audio playback (WebKitGTK trên Linux dev vs WebView2 trong bản Windows CI — có thể khác nhau), file `.exe` thật chỉ lấy được từ CI artifact.
- Thêm job `cargo test` + `vitest` vào `.github/workflows/build-windows.yml` trước bước build Tauri.

### Phase 8 — Follow-up (không block các phase trên)
- Crawl A1 content qua hub page (chưa xác định cấu trúc, cần khảo sát riêng).
- Tinh chỉnh mapping level theo show sau khi dùng thực tế.
- Xử lý contraction trong diff.

## File quan trọng sẽ sửa/tạo
- `src/utils/diff.ts`, `src/store/lessonStore.ts`, `src/store/progressStore.ts`, `src/services/tauri.ts`
- `src-tauri/src/db/mod.rs`, `src-tauri/src/db/migrations/0002_refine.sql`
- `src-tauri/src/scraper/{feeds,voa_rss,transcript}.rs`, `src-tauri/src/error.rs`
- `src-tauri/src/commands/{content,audio}.rs`, `src-tauri/src/lib.rs`
- `.github/workflows/build-windows.yml` (thêm test job)

## Verification
- Phase 1: chạy `npm run dev`, thao tác thủ công trên browser.
- Phase 2-4: `cargo test` với sqlite in-memory + fixture HTML/XML thật; gọi command qua Tauri dev console.
- Phase 5: `npx vitest run src/utils/diff.test.ts`.
- Phase 7: CI xanh trên GitHub Actions (cả job test và job build Windows).
