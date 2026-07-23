# English Listen

Ứng dụng luyện nghe – gõ chính tả (dictation) tiếng Anh, dùng nội dung từ [VOA Learning English](https://learningenglish.voanews.com/) (RSS + transcript + mp3 chính thức).

## Kiến trúc

- **Frontend**: React + TypeScript + Vite — audio player, ô gõ chính tả, so sánh diff, theo dõi tiến độ theo cấp độ A1/A2/B1.
- **Tauri Core (Rust)**: lưu dữ liệu local bằng SQLite, các command fetch nội dung mới và quản lý audio file offline.
- **Nguồn nội dung**: VOA Learning English.

## Cấu trúc thư mục

```
english_listen/
├── src/                    # Frontend (React)
│   ├── components/         # AudioPlayer, DictationInput, DiffViewer, ProgressTracker, LevelSelector
│   ├── pages/              # Home, Practice, Progress
│   ├── hooks/
│   ├── store/               # zustand stores
│   ├── services/            # wrappers gọi Tauri invoke()
│   ├── types/
│   ├── utils/                # diff.ts
│   └── styles/
├── src-tauri/               # Backend (Rust)
│   ├── src/
│   │   ├── commands/         # content.rs (fetch), audio.rs (offline files)
│   │   ├── db/                # models + SQLite migrations
│   │   └── scraper/           # voa_rss.rs, transcript.rs
│   ├── capabilities/
│   └── tauri.conf.json
├── docs/
└── scripts/
```

## Bắt đầu

Đây là bộ khung ban đầu (chưa cài dependencies). Để khởi tạo thật:

```bash
npm install
npm run tauri dev
```
