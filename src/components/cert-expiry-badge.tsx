import { getCertExpiryStatus } from "../types";

interface Props {
  valid_to: number;
  showDate?: boolean;
}

export default function CertExpiryBadge({ valid_to, showDate = false }: Props) {
  const status = getCertExpiryStatus(valid_to);
  const date = new Date(valid_to * 1000).toLocaleDateString();

  const badgeClass =
    status === "valid"         ? "badge badge-success" :
    status === "expiring_soon" ? "badge badge-warning" :
    "badge badge-error";

  const label =
    status === "valid"         ? "Valid" :
    status === "expiring_soon" ? "Expiring Soon" :
    "Expired";

  return (
    <span>
      <span className={badgeClass}>{label}</span>
      {showDate && (
        <span style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-muted-light)", marginLeft: 6 }}>
          {date}
        </span>
      )}
    </span>
  );
}
