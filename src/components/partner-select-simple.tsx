import { useEffect, useState } from "react";
import { listPartners } from "../lib/tauri-api";
import type { Partner } from "../types";

interface Props {
  value: string | null;
  onChange: (name: string) => void;
}

export default function PartnerSelectSimple({ value, onChange }: Props) {
  const [partners, setPartners] = useState<Partner[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    listPartners()
      .then(setPartners)
      .catch(() => setPartners([]))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div style={{ color: "var(--color-text-muted-light)", padding: 12, fontSize: "var(--font-size-sm)" }}>
        Loading partners…
      </div>
    );
  }

  if (partners.length === 0) {
    return (
      <div style={{ color: "var(--color-text-muted-light)", padding: 12, fontSize: "var(--font-size-sm)" }}>
        No partners available. Create one in the Partners page.
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <span className="section-title" style={{ marginBottom: 4, color: "var(--color-text-muted-light)" }}>
        Select Partner
      </span>
      <div
        className="table-container"
        style={{ flex: 1, overflowY: "auto" }}
      >
        {partners.map((p) => {
          const isSelected = value === p.name;
          return (
            <div
              key={p.id}
              onClick={() => onChange(p.name)}
              style={{
                padding: "8px 12px",
                cursor: "pointer",
                borderBottom: "1px solid var(--color-border-light)",
                borderLeft: isSelected ? "3px solid var(--color-accent-primary)" : "3px solid transparent",
                background: isSelected ? "#e0f2fe" : "var(--color-bg-table-row)",
                color: "var(--color-text-on-light)",
                fontSize: "var(--font-size-base)",
              }}
            >
              {p.name}
            </div>
          );
        })}
      </div>
    </div>
  );
}
