import type { ScanStatus } from "../../../types";

interface Props {
  status: ScanStatus;
  onScan: () => void;
}

export default function ScanButton({ status, onScan }: Props) {
  const scanning = status.type === "scanning";
  const label =
    scanning
      ? "Scanning…"
      : status.type === "done"
        ? "Scan Again"
        : status.type === "error"
          ? "Retry"
          : "Scan Token";

  const extraStyle =
    status.type === "error"
      ? { border: "1px solid var(--color-accent-danger)" }
      : {};

  return (
    <button
      className="btn btn-primary"
      onClick={onScan}
      disabled={scanning}
      style={{ ...extraStyle }}
    >
      {scanning ? "⠋ " : "🔍 "}
      {label}
    </button>
  );
}
