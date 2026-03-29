import type { TokenCertEntry } from "../../../types";
import CertificateDetail from "./CertificateDetail";

interface Props {
  entries: TokenCertEntry[];
  selectedCertId: string | null;
  senderCertSerial: string | null;
  onSelect: (certObjectId: string, slotId: number) => void;
  onExport: () => Promise<void>;
}

export default function CertificateTable({
  entries,
  selectedCertId,
  senderCertSerial,
  onSelect,
  onExport,
}: Props) {
  if (entries.length === 0) {
    return (
      <div
        style={{
          color: "var(--color-text-muted-light)",
          fontSize: "var(--font-size-sm)",
          padding: "12px 0",
        }}
      >
        No certificates found on token.
      </div>
    );
  }

  const selectedEntry = entries.find(
    (e) => e.certificate.object_id === selectedCertId,
  );

  return (
    <div>
      <div className="table-container">
        <table>
          <thead>
            <tr>
              <th>Common Name</th>
              <th>Organization</th>
              <th>Valid Until</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {entries.map(({ slot_id, certificate: cert }) => {
              const isSelected = cert.object_id === selectedCertId;
              const isSender = cert.serial_number === senderCertSerial;

              const rowStyle: React.CSSProperties = isSelected
                ? { borderLeft: "3px solid var(--color-accent-primary)", background: "var(--color-bg-table-hover)" }
                : {};
              const mutedStyle: React.CSSProperties = cert.is_expired
                ? { color: "var(--color-text-muted-light)" }
                : {};

              return (
                <tr
                  key={cert.object_id}
                  style={{ cursor: "pointer", ...rowStyle }}
                  onClick={() => onSelect(cert.object_id, slot_id)}
                >
                  <td
                    style={{
                      ...mutedStyle,
                      fontWeight: isSender ? "var(--font-weight-semibold)" : undefined,
                    }}
                  >
                    {isSender && (
                      <span
                        style={{
                          color: "var(--color-accent-success)",
                          marginRight: 4,
                        }}
                      >
                        ✓
                      </span>
                    )}
                    {cert.subject_cn}
                  </td>
                  <td style={mutedStyle}>{cert.subject_org || "—"}</td>
                  <td style={mutedStyle}>{cert.valid_until}</td>
                  <td>
                    {cert.is_expired ? (
                      <span
                        style={{
                          color: "var(--color-accent-danger)",
                          fontSize: "var(--font-size-xs)",
                        }}
                      >
                        ✗ Expired
                      </span>
                    ) : (
                      <span
                        style={{
                          color: "var(--color-accent-success)",
                          fontSize: "var(--font-size-xs)",
                        }}
                      >
                        ✓ Valid
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {selectedEntry && (
        <CertificateDetail cert={selectedEntry.certificate} onExport={onExport} />
      )}
    </div>
  );
}
