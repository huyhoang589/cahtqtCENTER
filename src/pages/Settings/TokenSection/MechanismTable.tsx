import type { MechanismDetail } from "../../../types";

interface Props {
  mechanisms: MechanismDetail[];
}

export default function MechanismTable({ mechanisms }: Props) {
  if (mechanisms.length === 0) return null;

  return (
    <div style={{ marginTop: 8 }}>
      <div
        style={{
          fontSize: "var(--font-size-xs)",
          color: "var(--color-text-muted-light)",
          marginBottom: 4,
          fontWeight: "var(--font-weight-semibold)",
        }}
      >
        RSA Mechanism Support
      </div>
      <table
        style={{
          width: "100%",
          borderCollapse: "collapse",
          fontSize: "var(--font-size-xs)",
          color: "var(--color-text-on-light)",
        }}
      >
        <thead>
          <tr style={{ borderBottom: "1px solid var(--color-border-light)" }}>
            <th style={thStyle}>Mechanism</th>
            <th style={thStyle}>Standard</th>
            <th style={thStyle}>Key Size</th>
            <th style={thStyle}>Operations</th>
          </tr>
        </thead>
        <tbody>
          {mechanisms.map((m) => (
            <tr key={m.name} style={{ borderBottom: "1px solid var(--color-border-light)" }}>
              {/* Mechanism name + support indicator */}
              <td style={tdStyle}>
                <span
                  style={{
                    display: "inline-block",
                    width: 6,
                    height: 6,
                    borderRadius: "50%",
                    background: m.supported
                      ? "var(--color-accent-success)"
                      : "var(--color-accent-danger)",
                    marginRight: 6,
                    verticalAlign: "middle",
                  }}
                />
                {m.name}
              </td>
              <td style={tdStyle}>{m.pkcs_standard}</td>
              {/* Key size range or "—" if unsupported */}
              <td style={{ ...tdStyle, color: m.supported ? undefined : "var(--color-accent-danger)" }}>
                {m.supported ? `${m.min_key_bits}–${m.max_key_bits} bits` : "—"}
              </td>
              {/* Operations list or "Not supported" */}
              <td style={{ ...tdStyle, color: m.supported ? undefined : "var(--color-accent-danger)" }}>
                {m.supported ? (m.flags.length > 0 ? m.flags.join(", ") : "—") : "Not supported"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

const thStyle: React.CSSProperties = {
  textAlign: "left",
  padding: "4px 8px",
  color: "var(--color-text-muted-light)",
  fontWeight: "var(--font-weight-semibold)",
};

const tdStyle: React.CSSProperties = {
  padding: "4px 8px",
};
