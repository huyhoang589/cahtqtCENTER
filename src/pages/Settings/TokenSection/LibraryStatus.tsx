import type { LibraryInfo } from "../../../types";

interface Props {
  libraryInfo: LibraryInfo | null;
}

export default function LibraryStatus({ libraryInfo }: Props) {
  if (!libraryInfo) {
    return (
      <div style={{ color: "var(--color-text-muted-light)", fontSize: "var(--font-size-sm)" }}>
        No PKCS#11 middleware detected
      </div>
    );
  }
  return (
    <div
      style={{
        fontSize: "var(--font-size-sm)",
        display: "flex",
        flexDirection: "column",
        gap: 4,
      }}
    >
      <div>
        <span style={{ color: "var(--color-accent-success)" }}>●</span>
        <span style={{ marginLeft: 6, color: "var(--color-text-on-light)" }}>
          {libraryInfo.vendor} — {libraryInfo.description || "PKCS#11 Library"}
        </span>
      </div>
      <div
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--font-size-xs)",
          color: "var(--color-text-muted-light)",
        }}
      >
        {libraryInfo.path}
      </div>
      <div style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-muted-light)" }}>
        v{libraryInfo.library_version} · PKCS#11 {libraryInfo.cryptoki_version}
      </div>
    </div>
  );
}
