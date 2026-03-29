import { useEffect, useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { listPartners, listPartnerMembers } from "../lib/tauri-api";
import type { Partner, PartnerMember } from "../types";
import PartnerMemberRow from "./partner-member-row";

interface Props {
  selectedIds: string[];
  /// Also emits certPaths (cert_file_path per member) and partnerName (first partner with selections).
  onSelectionChange: (ids: string[], certPaths: string[], partnerName: string) => void;
}

export default function PartnerSelectPanel({ selectedIds, onSelectionChange }: Props) {
  const [partners, setPartners] = useState<Partner[]>([]);
  const [members, setMembers] = useState<Record<string, PartnerMember[]>>({});
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(true);

  useEffect(() => { loadPartners(); }, []);

  const loadPartners = async () => {
    try {
      setLoading(true);
      const ps = await listPartners();
      setPartners(ps);
      if (ps.length > 0) setExpanded(new Set([ps[0].id]));
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  const toggleExpand = async (partnerId: string) => {
    const next = new Set(expanded);
    if (next.has(partnerId)) {
      next.delete(partnerId);
    } else {
      next.add(partnerId);
      if (!members[partnerId]) {
        try {
          const ms = await listPartnerMembers(partnerId);
          setMembers((prev) => ({ ...prev, [partnerId]: ms }));
        } catch { /* ignore */ }
      }
    }
    setExpanded(next);
  };

  /// Derive certPaths and partnerName from the updated selection.
  const buildSelectionData = (newIds: string[]) => {
    const allMembers = Object.values(members).flat();
    const certPaths = newIds
      .map((id) => allMembers.find((m) => m.id === id)?.cert_file_path)
      .filter((cp): cp is string => !!cp && cp.length > 0);
    const firstPartner = partners.find((p) =>
      (members[p.id] ?? []).some((m) => newIds.includes(m.id))
    );
    return { certPaths, partnerName: firstPartner?.name ?? "" };
  };

  const toggleMember = (id: string) => {
    const newIds = selectedIds.includes(id)
      ? selectedIds.filter((x) => x !== id)
      : [...selectedIds, id];
    const { certPaths, partnerName } = buildSelectionData(newIds);
    onSelectionChange(newIds, certPaths, partnerName);
  };

  const selectAllInPartner = (partnerId: string) => {
    const ids = (members[partnerId] ?? []).map((m) => m.id);
    const newIds = [...new Set([...selectedIds, ...ids])];
    const { certPaths, partnerName } = buildSelectionData(newIds);
    onSelectionChange(newIds, certPaths, partnerName);
  };

  const deselectAllInPartner = (partnerId: string) => {
    const ids = new Set((members[partnerId] ?? []).map((m) => m.id));
    const newIds = selectedIds.filter((id) => !ids.has(id));
    const { certPaths, partnerName } = buildSelectionData(newIds);
    onSelectionChange(newIds, certPaths, partnerName);
  };

  if (loading) return <div style={{ color: "var(--color-text-muted-light)", padding: 12 }}>Loading partners…</div>;
  if (partners.length === 0) return (
    <div style={{ color: "var(--color-text-muted-light)", padding: 12, fontSize: "var(--font-size-sm)" }}>
      No partners. Go to Partners to add partner members.
    </div>
  );

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span className="section-title" style={{ marginBottom: 0, color: "var(--color-text-muted-light)" }}>
          Partners ({selectedIds.length} selected)
        </span>
        <button
          className="btn btn-ghost"
          onClick={() => onSelectionChange([], [], "")}
          style={{ height: 24, padding: "0 8px", fontSize: "var(--font-size-xs)" }}
        >
          Clear
        </button>
      </div>

      <div
        className="table-container"
        style={{ flex: 1, overflowY: "auto" }}
      >
        {partners.map((p) => {
          const isOpen = expanded.has(p.id);
          const ms = members[p.id] ?? [];
          const partnerSelectedCount = ms.filter((m) => selectedIds.includes(m.id)).length;

          return (
            <div key={p.id}>
              {/* Partner header */}
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  padding: "7px 10px",
                  cursor: "pointer",
                  background: "var(--color-bg-table-header)",
                  borderBottom: "1px solid var(--color-border-dark)",
                  gap: 6,
                }}
                onClick={() => toggleExpand(p.id)}
              >
                {isOpen ? <ChevronDown size={12} color="var(--color-text-secondary)" /> : <ChevronRight size={12} color="var(--color-text-secondary)" />}
                <span style={{ flex: 1, fontWeight: "var(--font-weight-semibold)", color: "var(--color-text-primary)", fontSize: "var(--font-size-sm)" }}>
                  {p.name}
                </span>
                <span style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)" }}>
                  {partnerSelectedCount}/{p.member_count ?? 0}
                </span>
                {isOpen && ms.length > 0 && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      partnerSelectedCount === ms.length ? deselectAllInPartner(p.id) : selectAllInPartner(p.id);
                    }}
                    style={{
                      fontSize: "var(--font-size-xs)",
                      padding: "1px 6px",
                      background: "rgba(0,180,216,0.15)",
                      color: "var(--color-accent-primary)",
                      borderRadius: "var(--radius-sm)",
                      border: "none",
                      cursor: "pointer",
                    }}
                  >
                    {partnerSelectedCount === ms.length ? "None" : "All"}
                  </button>
                )}
              </div>

              {/* Member table headers + rows */}
              {isOpen && ms.length > 0 && (
                // A-1: horizontal scroll for narrow panels
                <div style={{ overflowX: "auto", width: "100%" }}>
                  <div style={{ minWidth: 340 }}>
                    <div style={{ display: "flex", alignItems: "center", padding: "3px 10px 3px 24px", gap: 8, background: "var(--color-bg-table-header)", borderBottom: "1px solid var(--color-border-dark)" }}>
                      <span style={{ width: 16, flexShrink: 0 }} />
                      <span style={{ flex: 2, fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)", fontWeight: 600 }}>Name</span>
                      <span style={{ flex: 2, fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)", fontWeight: 600 }}>Organization</span>
                      <span style={{ flexShrink: 0, fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)", fontWeight: 600 }}>Expires</span>
                    </div>
                    {ms.map((m) => (
                      <PartnerMemberRow
                        key={m.id}
                        member={m}
                        checked={selectedIds.includes(m.id)}
                        onToggle={() => toggleMember(m.id)}
                      />
                    ))}
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
