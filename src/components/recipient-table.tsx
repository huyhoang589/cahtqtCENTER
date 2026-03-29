import type { PartnerMember } from "../types";
import { deletePartnerMember } from "../lib/tauri-api";
import CertExpiryBadge from "./cert-expiry-badge";
import MemberActionButtons from "./member-action-buttons";

interface Props {
  recipients: PartnerMember[];
  onRefresh: () => void;
  onAddRecipient: () => void;
  onRowSelect?: (id: string) => void;
  selectedMemberId?: string | null;
  partnerName: string;
  outputDataDir: string;
  desktopPath: string;
}

export default function RecipientTable({
  recipients, onRefresh, onAddRecipient, onRowSelect, selectedMemberId,
  partnerName, outputDataDir, desktopPath,
}: Props) {
  const handleDelete = async (id: string, name: string) => {
    if (!window.confirm(`Remove partner member "${name}"?`)) return;
    try {
      await deletePartnerMember(id);
      onRefresh();
    } catch (e) {
      alert(`Error: ${e}`);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Toolbar */}
      <div style={{ padding: "10px 16px", borderBottom: "1px solid var(--color-border-light)", display: "flex", justifyContent: "space-between", alignItems: "center", background: "var(--color-bg-content)" }}>
        <span className="section-title" style={{ marginBottom: 0, color: "var(--color-text-muted-light)" }}>
          Partner Members ({recipients.length})
        </span>
        <button className="btn btn-primary" onClick={onAddRecipient}>+ Add Partner Member</button>
      </div>

      {/* Table */}
      <div style={{ flex: 1, overflowY: "auto", background: "var(--color-bg-content)" }}>
        {recipients.length === 0 ? (
          <div style={{ padding: 24, color: "var(--color-text-muted-light)", textAlign: "center" }}>
            No partner members. Click "+ Add Partner Member" to import a certificate.
          </div>
        ) : (
          <div className="table-container" style={{ margin: 16 }}>
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>CN</th>
                  <th>Organization</th>
                  <th>Expires</th>
                  <th style={{ width: 160 }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {recipients.map((r) => (
                  <tr
                    key={r.id}
                    onClick={() => onRowSelect?.(r.id)}
                    style={{ cursor: onRowSelect ? "pointer" : undefined, background: selectedMemberId === r.id ? "#e0f2fe" : undefined }}
                  >
                    <td style={{ color: "var(--color-text-on-light)" }}>{r.name}</td>
                    <td style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-on-light)" }}>{r.cert_cn}</td>
                    <td style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-muted-light)" }}>{r.cert_org ?? "—"}</td>
                    <td><CertExpiryBadge valid_to={r.cert_valid_to} showDate /></td>
                    <td onClick={(e) => e.stopPropagation()}>
                      <MemberActionButtons
                        member={r}
                        partnerName={partnerName}
                        outputDataDir={outputDataDir}
                        desktopPath={desktopPath}
                        onRemove={() => handleDelete(r.id, r.name)}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
