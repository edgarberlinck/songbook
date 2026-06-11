import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import "./App.css";

type SongSummary = {
  id: string;
  title: string;
  artist: string | null;
  key: string | null;
  favorite: boolean;
  lastModified: number;
  tags: string[];
};

type RenderedLine = {
  kind: "section" | "meta" | "lyric";
  label: string | null;
  chordLine: string | null;
  lyricLine: string | null;
  chorus: boolean;
};

type SongDetail = SongSummary & {
  subtitle: string | null;
  album: string | null;
  capo: number | null;
  tempo: number | null;
  notes: string | null;
  createdAt: number;
  content: string;
  preview: RenderedLine[];
};

type LibraryPayload = {
  songs: SongSummary[];
  availableTags: string[];
};

const emptyState = {
  songs: [],
  availableTags: [],
} satisfies LibraryPayload;

const sorters: Record<string, (a: SongSummary, b: SongSummary) => number> = {
  recent: (a, b) => b.lastModified - a.lastModified,
  title: (a, b) => a.title.localeCompare(b.title),
  artist: (a, b) => (a.artist ?? "").localeCompare(b.artist ?? ""),
};

function formatDate(timestamp: number) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
  }).format(new Date(timestamp * 1000));
}

function detectProvider(url: string): { name: string; supported: boolean } {
  const normalized = url.trim().toLowerCase();
  if (!normalized) {
    return { name: "Unknown", supported: false };
  }
  if (normalized.includes("cifraclub.com")) {
    return { name: "Cifra Club", supported: true };
  }
  return { name: "Unknown", supported: false };
}

function App() {
  const [library, setLibrary] = useState<LibraryPayload>(emptyState);
  const [activeSong, setActiveSong] = useState<SongDetail | null>(null);
  const [previewSong, setPreviewSong] = useState<SongDetail | null>(null);
  const [draft, setDraft] = useState("");
  const [query, setQuery] = useState("");
  const [selectedTag, setSelectedTag] = useState<string>("all");
  const [showFavoritesOnly, setShowFavoritesOnly] = useState(false);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<keyof typeof sorters>("recent");
  const [showChords, setShowChords] = useState(true);
  const [readingMode, setReadingMode] = useState(false);
  const [fontScale, setFontScale] = useState(1);
  const [theme, setTheme] = useState<"light" | "dark">("dark");
  const [status, setStatus] = useState("Loading library…");
  const [showImportModal, setShowImportModal] = useState(false);
  const [importUrl, setImportUrl] = useState("");
  const [importError, setImportError] = useState<string | null>(null);
  const [importLoading, setImportLoading] = useState(false);
  const editorScrollRef = useRef<HTMLTextAreaElement | null>(null);
  const previewScrollRef = useRef<HTMLDivElement | null>(null);
  const syncOriginRef = useRef<"editor" | "preview" | null>(null);

  const importProvider = useMemo(() => detectProvider(importUrl), [importUrl]);

  const syncScrollPosition = useCallback(
    (
      origin: "editor" | "preview",
      source: HTMLElement,
      target: HTMLElement,
    ) => {
      if (syncOriginRef.current && syncOriginRef.current !== origin) {
        return;
      }

      syncOriginRef.current = origin;
      const sourceMax = source.scrollHeight - source.clientHeight;
      const targetMax = target.scrollHeight - target.clientHeight;

      if (sourceMax <= 0 || targetMax <= 0) {
        target.scrollTop = 0;
      } else {
        const ratio = source.scrollTop / sourceMax;
        target.scrollTop = ratio * targetMax;
      }

      requestAnimationFrame(() => {
        if (syncOriginRef.current === origin) {
          syncOriginRef.current = null;
        }
      });
    },
    [],
  );

  const handleEditorScroll = useCallback(() => {
    const editor = editorScrollRef.current;
    const preview = previewScrollRef.current;
    if (!editor || !preview) {
      return;
    }
    syncScrollPosition("editor", editor, preview);
  }, [syncScrollPosition]);

  const handlePreviewScroll = useCallback(() => {
    const editor = editorScrollRef.current;
    const preview = previewScrollRef.current;
    if (!editor || !preview) {
      return;
    }
    syncScrollPosition("preview", preview, editor);
  }, [syncScrollPosition]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const refreshLibrary = useCallback(
    async (preferredId?: string) => {
      const payload = await invoke<LibraryPayload>("load_library");
      setLibrary(payload);
      const nextId = preferredId ?? activeSong?.id ?? payload.songs[0]?.id;
      if (nextId) {
        const song = await invoke<SongDetail>("load_song", { id: nextId });
        setActiveSong(song);
        setPreviewSong(song);
        setDraft(song.content);
        setSelectedIds((current) => (current.length ? current : [song.id]));
        setStatus(`Loaded ${payload.songs.length} songs`);
      } else {
        setActiveSong(null);
        setPreviewSong(null);
        setDraft("");
        setStatus("No songs found in songs/");
      }
    },
    [activeSong?.id],
  );

  const toggleFavoriteFilter = () => {
    setShowFavoritesOnly((current) => !current);
  };

  const toggleTagFilter = (tag: string) => {
    setSelectedTag((current) => (current === tag ? "all" : tag));
  };

  async function handleCreateSong() {
    const song = await invoke<SongDetail>("create_song");
    setStatus(`Created ${song.title}`);
    setSelectedIds([song.id]);
    await refreshLibrary(song.id);
  }

  async function handleImportSong() {
    if (!importUrl.trim()) {
      setImportError("Paste a URL before importing");
      return;
    }

    if (!importProvider.supported) {
      setImportError("This provider is not supported yet");
      return;
    }

    setImportLoading(true);
    setImportError(null);

    try {
      const song = await invoke<SongDetail>("import_song_from_url", {
        url: importUrl.trim(),
      });
      setStatus(`Imported ${song.title}`);
      setSelectedIds([song.id]);
      setShowImportModal(false);
      setImportUrl("");
      await refreshLibrary(song.id);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setImportError(message);
    } finally {
      setImportLoading(false);
    }
  }

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      void refreshLibrary();
    }, 0);

    return () => window.clearTimeout(timeoutId);
  }, [refreshLibrary]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = event.metaKey || event.ctrlKey;
      if (modifier && event.key.toLowerCase() === "s") {
        event.preventDefault();
        void handleSave();
      }
      if (modifier && event.key.toLowerCase() === "b") {
        event.preventDefault();
        setShowChords((value) => !value);
      }
      if (modifier && event.key === "=") {
        event.preventDefault();
        setFontScale((value) => Math.min(1.8, value + 0.1));
      }
      if (modifier && event.key === "-") {
        event.preventDefault();
        setFontScale((value) => Math.max(0.8, value - 0.1));
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

  const filteredSongs = useMemo(() => {
    return [...library.songs]
      .filter((song) => !showFavoritesOnly || song.favorite)
      .filter(
        (song) => selectedTag === "all" || song.tags.includes(selectedTag),
      )
      .filter((song) => {
        const haystack = [
          song.title,
          song.artist ?? "",
          song.key ?? "",
          song.tags.join(" "),
        ]
          .join(" ")
          .toLowerCase();
        return haystack.includes(query.trim().toLowerCase());
      })
      .sort(sorters[sortBy]);
  }, [library.songs, query, selectedTag, showFavoritesOnly, sortBy]);

  async function selectSong(id: string) {
    const song = await invoke<SongDetail>("load_song", { id });
    setActiveSong(song);
    setPreviewSong(song);
    setDraft(song.content);
    setSelectedIds((current) => (current.includes(id) ? current : [id]));
    setStatus(`Opened ${song.title}`);
  }

  async function handleSave() {
    if (!activeSong) return;
    const song = await invoke<SongDetail>("save_song", {
      id: activeSong.id,
      content: draft,
    });
    setActiveSong(song);
    setPreviewSong(song);
    setDraft(song.content);
    setStatus(`Saved ${song.title}`);
    await refreshLibrary(song.id);
  }

  async function handleTranspose(semitones: number) {
    const transposed = await invoke<string>("transpose_content", {
      content: draft,
      semitones,
    });
    setDraft(transposed);
    const unit = Math.abs(semitones) === 1 ? "semitone" : "semitones";
    setStatus(`Transposed ${semitones > 0 ? "+" : ""}${semitones} ${unit}`);
  }

  const displayedSong =
    activeSong && draft === activeSong.content ? activeSong : previewSong;
  const preview = displayedSong?.preview ?? [];

  useEffect(() => {
    if (!activeSong || draft === activeSong.content) return;

    const timeoutId = window.setTimeout(async () => {
      const song = await invoke<SongDetail>("save_song", {
        id: activeSong.id,
        content: draft,
      });
      setActiveSong(song);
      setPreviewSong(song);
      setStatus(`Autosaved ${song.title}`);
      await refreshLibrary(song.id);
    }, 900);

    return () => window.clearTimeout(timeoutId);
  }, [activeSong, draft, refreshLibrary]);

  useEffect(() => {
    if (!activeSong || draft === activeSong.content) return;

    const timeoutId = window.setTimeout(async () => {
      const preview = await invoke<SongDetail>("preview_song", {
        id: activeSong.id,
        content: draft,
      });
      setPreviewSong(preview);
    }, 120);

    return () => window.clearTimeout(timeoutId);
  }, [activeSong, draft]);

  return (
    <>
      <main className={`app-shell ${readingMode ? "reading-mode" : ""}`}>
        <aside className="sidebar">
          <div className="brand">
            <div>♪</div>
            <div>
              <strong>Songbook</strong>
              <span>Local-first ChordPro library</span>
            </div>
          </div>

          <div className="segmented">
            <button
              className={!showFavoritesOnly ? "active" : ""}
              onClick={() => setShowFavoritesOnly(false)}
            >
              All Songs
            </button>
            <button
              className={showFavoritesOnly ? "active" : ""}
              onClick={toggleFavoriteFilter}
            >
              Favorites
            </button>
          </div>

          <label className="search">
            <span>Search</span>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Title, artist, key, tag"
            />
          </label>

          <div className="controls-row">
            <label>
              <span>Sort</span>
              <select
                value={sortBy}
                onChange={(event) =>
                  setSortBy(event.target.value as keyof typeof sorters)
                }
              >
                <option value="recent">Recently Modified</option>
                <option value="title">Title</option>
                <option value="artist">Artist</option>
              </select>
            </label>
            <label>
              <span>Tag</span>
              <select
                value={selectedTag}
                onChange={(event) => setSelectedTag(event.target.value)}
              >
                <option value="all">All tags</option>
                {library.availableTags.map((tag) => (
                  <option key={tag} value={tag}>
                    {tag}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <section className="library-section">
            <header>
              <strong>Library</strong>
              <span>{filteredSongs.length} songs</span>
            </header>
            <ul className="song-list">
              {filteredSongs.map((song) => {
                const active = activeSong?.id === song.id;
                const selected = selectedIds.includes(song.id);
                return (
                  <li key={song.id}>
                    <button
                      className={`song-card ${active ? "active" : ""}`}
                      onClick={() => void selectSong(song.id)}
                    >
                      <input
                        type="checkbox"
                        checked={selected}
                        onChange={(event) => {
                          event.stopPropagation();
                          setSelectedIds((current) =>
                            event.target.checked
                              ? [...new Set([...current, song.id])]
                              : current.filter((item) => item !== song.id),
                          );
                        }}
                      />
                      <div>
                        <strong>{song.title}</strong>
                        <span>{song.artist ?? "Unknown artist"}</span>
                      </div>
                      <small>
                        {song.key ?? "—"} · {formatDate(song.lastModified)}
                      </small>
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>

          <section className="tags-section">
            <header>Tags</header>
            <div className="tags">
              {library.availableTags.map((tag) => (
                <button
                  key={tag}
                  className={selectedTag === tag ? "active" : ""}
                  onClick={() => toggleTagFilter(tag)}
                >
                  {tag}
                </button>
              ))}
            </div>
          </section>
        </aside>

        <section className="workspace">
          <header className="toolbar">
            <div>
              <h1>{activeSong?.title ?? "Songbook"}</h1>
              <p>
                {activeSong?.artist ?? "Distraction-free writing and rehearsal"}
              </p>
            </div>
            <div className="toolbar-actions">
              <button
                className="primary"
                onClick={() => void handleCreateSong()}
              >
                New song
              </button>
              <button onClick={() => setShowImportModal(true)}>
                Import song
              </button>
              <button onClick={() => void handleTranspose(-1)}>−1</button>
              <button onClick={() => void handleTranspose(1)}>+1</button>
              <button onClick={() => setShowChords((value) => !value)}>
                {showChords ? "Hide chords" : "Show chords"}
              </button>
              <button onClick={() => setReadingMode((value) => !value)}>
                {readingMode ? "Editor mode" : "Reading mode"}
              </button>
              <button
                onClick={() =>
                  setTheme((value) => (value === "dark" ? "light" : "dark"))
                }
              >
                {theme === "dark" ? "Light" : "Dark"} mode
              </button>
              <button className="primary" onClick={() => void handleSave()}>
                Save
              </button>
            </div>
          </header>

          <div className="meta-strip">
            <span>Selected: {selectedIds.length}</span>
            <span>Capo: {activeSong?.capo ?? "—"}</span>
            <span>Tempo: {activeSong?.tempo ?? "—"}</span>
            <span>Zoom: {(fontScale * 100).toFixed(0)}%</span>
            <span>{status}</span>
          </div>

          <div className="split-view">
            <section className="pane editor-pane">
              <header>ChordPro editor</header>
              <textarea
                ref={editorScrollRef}
                onScroll={handleEditorScroll}
                spellCheck={false}
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                placeholder="Write your song in ChordPro…"
              />
            </section>

            <section
              className="pane preview-pane"
              style={{ fontSize: `${fontScale}rem` }}
            >
              <header>Live preview</header>
              <div
                className="preview-scroll"
                ref={previewScrollRef}
                onScroll={handlePreviewScroll}
              >
                {displayedSong && (
                  <article>
                    <div className="preview-heading">
                      <h2>{displayedSong.title}</h2>
                      {displayedSong.subtitle ? (
                        <p>{displayedSong.subtitle}</p>
                      ) : null}
                      <div className="chips">
                        {displayedSong.key ? (
                          <span>Key {displayedSong.key}</span>
                        ) : null}
                        {displayedSong.capo !== null ? (
                          <span>Capo {displayedSong.capo}</span>
                        ) : null}
                        {displayedSong.favorite ? <span>Favorite</span> : null}
                      </div>
                    </div>

                    {preview.map((line, index) => (
                      <div
                        key={`${index}-${line.label ?? line.lyricLine ?? "line"}`}
                        className={`rendered-line ${line.kind} ${line.chorus ? "chorus" : ""}`}
                      >
                        {line.kind === "section" ? <h3>{line.label}</h3> : null}
                        {line.kind === "meta" ? (
                          <p className="meta-line">{line.label}</p>
                        ) : null}
                        {line.kind === "lyric" ? (
                          <div className="lyric-stack">
                            {showChords && line.chordLine ? (
                              <div
                                className="chord-line"
                                aria-label="Chord line"
                              >
                                {line.chordLine}
                              </div>
                            ) : null}
                            <div className="lyric-line">{line.lyricLine}</div>
                          </div>
                        ) : null}
                      </div>
                    ))}
                  </article>
                )}
              </div>
            </section>
          </div>
        </section>
      </main>

      {showImportModal ? (
        <div
          className="modal-backdrop"
          onClick={() => setShowImportModal(false)}
        >
          <section
            className="import-modal"
            onClick={(event) => event.stopPropagation()}
          >
            <header>
              <h2>Import song from URL</h2>
              <p>Paste a song URL to import.</p>
            </header>

            <label className="search">
              <span>URL</span>
              <input
                value={importUrl}
                onChange={(event) => setImportUrl(event.target.value)}
                placeholder="https://www.cifraclub.com.br/..."
              />
            </label>

            <p
              className={`provider-status ${importProvider.supported ? "supported" : "unsupported"}`}
            >
              Provider: {importProvider.name} (
              {importProvider.supported ? "supported" : "unsupported"})
            </p>

            {importError ? <p className="import-error">{importError}</p> : null}

            <div className="modal-actions">
              <button
                onClick={() => {
                  setShowImportModal(false);
                  setImportError(null);
                }}
                disabled={importLoading}
              >
                Cancel
              </button>
              <button
                className="primary"
                onClick={() => void handleImportSong()}
                disabled={importLoading || !importProvider.supported}
              >
                {importLoading ? "Importing..." : "Import"}
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </>
  );
}

export default App;
