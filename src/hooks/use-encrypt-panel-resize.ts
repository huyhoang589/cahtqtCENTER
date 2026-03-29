import { useCallback, useEffect, useRef, useState } from "react";
import { getSettings, setSetting } from "../lib/tauri-api";

export function useEncryptPanelResize(defaultRatio = 0.55) {
  const [displayRatio, setDisplayRatio] = useState(defaultRatio);
  const containerRef = useRef<HTMLDivElement>(null);
  const leftRatioRef = useRef(defaultRatio);
  const isDragging = useRef(false);

  // Stable refs for mousemove/mouseup handlers — avoids stale closure mismatch
  // between addEventListener and removeEventListener calls
  const handleMouseMoveRef = useRef<(e: MouseEvent) => void>(() => {});
  const handleMouseUpRef = useRef<() => void>(() => {});

  // Stable wrappers defined once — identity never changes across renders
  const stableMouseMove = useCallback((e: MouseEvent) => handleMouseMoveRef.current(e), []);
  const stableMouseUp = useCallback(() => handleMouseUpRef.current(), []);

  // Update refs on each render so handlers always see fresh closure state
  handleMouseMoveRef.current = (e: MouseEvent) => {
    if (!isDragging.current || !containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const raw = (e.clientX - rect.left) / rect.width;
    const minLeft = 320 / rect.width;
    const minRight = 260 / rect.width;
    const clamped = Math.min(Math.max(raw, minLeft), 1 - minRight);
    leftRatioRef.current = clamped;
    const leftEl = containerRef.current.querySelector<HTMLElement>(".encrypt-split-left");
    const rightEl = containerRef.current.querySelector<HTMLElement>(".encrypt-split-right");
    if (leftEl) leftEl.style.width = `${clamped * 100}%`;
    if (rightEl) rightEl.style.width = `${(1 - clamped) * 100}%`;
  };

  handleMouseUpRef.current = () => {
    isDragging.current = false;
    document.removeEventListener("mousemove", stableMouseMove);
    document.removeEventListener("mouseup", stableMouseUp);
    setDisplayRatio(leftRatioRef.current);
    setSetting("encrypt_panel_split_ratio", leftRatioRef.current.toString()).catch(() => {});
  };

  // Restore persisted ratio on mount
  useEffect(() => {
    getSettings().then((settings) => {
      const saved = parseFloat(settings["encrypt_panel_split_ratio"] ?? String(defaultRatio));
      if (!isNaN(saved) && saved > 0 && saved < 1) {
        leftRatioRef.current = saved;
        setDisplayRatio(saved);
      }
    }).catch(() => {});
  }, []);

  const onDividerMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    document.addEventListener("mousemove", stableMouseMove);
    document.addEventListener("mouseup", stableMouseUp);
  };

  return { displayRatio, containerRef, onDividerMouseDown };
}
