import { useEffect, useRef, useState } from "react";
import { X } from "lucide-react";
import type { LogEntry } from "../hooks/use-log-panel";

interface Props {
  entries: LogEntry[];
  onClear: () => void;
}

const LEVEL_COLOR: Record<LogEntry["level"], string> = {
  info:    "var(--color-text-log-info)",
  success: "var(--color-text-log-success)",
  warning: "var(--color-text-log-warning)",
  error:   "var(--color-text-log-error)",
};

const STORED_HEIGHT_KEY = "log-panel-height";
const DEFAULT_HEIGHT = 140;
const MIN_HEIGHT = 80;
const MAX_HEIGHT = 400;

export default function LogPanel({ entries, onClear }: Props) {
  const [height, setHeight] = useState<number>(() => {
    const stored = localStorage.getItem(STORED_HEIGHT_KEY);
    return stored ? parseInt(stored, 10) : DEFAULT_HEIGHT;
  });
  const containerRef = useRef<HTMLDivElement>(null);
  const isDragging = useRef(false);
  const startY = useRef(0);
  const startHeight = useRef(0);

  // Auto-scroll to bottom on new entries
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [entries]);

  const onMouseDown = (e: React.MouseEvent) => {
    isDragging.current = true;
    startY.current = e.clientY;
    startHeight.current = height;
    e.preventDefault();
  };

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      const delta = startY.current - e.clientY;
      const newH = Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, startHeight.current + delta));
      setHeight(newH);
      localStorage.setItem(STORED_HEIGHT_KEY, String(newH));
    };
    const onUp = () => { isDragging.current = false; };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, []);

  return (
    <div
      style={{
        height,
        flexShrink: 0,
        background: "var(--color-bg-log)",
        borderTop: "1px solid var(--color-border-dark)",
        position: "relative",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Resize handle */}
      <div
        onMouseDown={onMouseDown}
        style={{
          height: 4,
          cursor: "row-resize",
          background: "transparent",
          flexShrink: 0,
        }}
      />

      {/* Clear button */}
      <button
        className="btn-icon"
        onClick={onClear}
        title="Clear log"
        style={{ position: "absolute", top: 4, right: 8, zIndex: 1 }}
      >
        <X size={14} />
      </button>

      {/* Log entries */}
      <div
        ref={containerRef}
        style={{
          flex: 1,
          overflowY: "auto",
          padding: "4px 16px 8px",
          fontFamily: "var(--font-mono)",
          fontSize: "var(--font-size-sm)",
        }}
      >
        {entries.length === 0 ? (
          <span style={{ color: "var(--color-text-secondary)" }}>No log entries yet.</span>
        ) : (
          entries.map((entry) => (
            <div key={entry.id} style={{ lineHeight: "var(--line-height-base)" }}>
              <span style={{ color: "var(--color-text-secondary)" }}>[{entry.timestamp}]</span>{" "}
              <span
                style={{
                  color: LEVEL_COLOR[entry.level],
                  fontWeight: "var(--font-weight-medium)",
                }}
              >
                [{entry.level.toUpperCase()}]
              </span>{" "}
              <span style={{ color: "var(--color-text-primary)" }}>{entry.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
