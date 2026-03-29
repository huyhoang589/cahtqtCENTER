import * as Dialog from "@radix-ui/react-dialog";

interface Props {
  fileCount: number;
  recipientCount: number;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmEncryptDialog({
  fileCount,
  recipientCount,
  onConfirm,
  onCancel,
}: Props) {
  const total = fileCount * recipientCount;

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open) onCancel(); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content className="dialog-content" aria-describedby={undefined}>
          <div className="dialog-header">
            <Dialog.Title className="dialog-title">Confirm Encryption</Dialog.Title>
          </div>
          <div className="dialog-body">
            <p style={{ color: "var(--color-text-secondary)", marginBottom: 12, lineHeight: "var(--line-height-base)" }}>
              Encrypt{" "}
              <strong style={{ color: "var(--color-text-primary)" }}>
                {fileCount} file{fileCount !== 1 ? "s" : ""}
              </strong>{" "}
              for{" "}
              <strong style={{ color: "var(--color-text-primary)" }}>
                {recipientCount} partner{recipientCount !== 1 ? "s" : ""}
              </strong>.
            </p>
            <p style={{ color: "var(--color-text-secondary)", fontSize: "var(--font-size-sm)" }}>
              This will create{" "}
              <strong style={{ color: "var(--color-accent-primary)" }}>{total}</strong>{" "}
              encrypted file{total !== 1 ? "s" : ""}.
            </p>
          </div>
          <div className="dialog-footer">
            <button className="btn btn-ghost" onClick={onCancel}>Cancel</button>
            <button className="btn btn-primary" onClick={onConfirm}>Proceed</button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
