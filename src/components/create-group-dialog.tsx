import { useEffect, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";

interface Props {
  onConfirm: (name: string) => void;
  onCancel: () => void;
}

export default function CreateGroupDialog({ onConfirm, onCancel }: Props) {
  const [name, setName] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const t = setTimeout(() => inputRef.current?.focus(), 50);
    return () => clearTimeout(t);
  }, []);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) onConfirm(name.trim());
  };

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open) onCancel(); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" />
        <Dialog.Content
          className="dialog-content"
          style={{ width: 320 }}
          aria-describedby={undefined}
        >
          <div className="dialog-header">
            <Dialog.Title className="dialog-title">Create Partner</Dialog.Title>
          </div>
          <div className="dialog-body">
            <form id="create-group-form" onSubmit={submit}>
              <input
                ref={inputRef}
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Partner name"
              />
            </form>
          </div>
          <div className="dialog-footer">
            <button className="btn btn-ghost" type="button" onClick={onCancel}>Cancel</button>
            <button
              className="btn btn-primary"
              type="submit"
              form="create-group-form"
              disabled={!name.trim()}
            >
              Create
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
