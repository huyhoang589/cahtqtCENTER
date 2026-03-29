import { useEffect, useRef, useState } from "react";
import AddRecipientDialog from "../components/add-recipient-dialog";
import GroupListSidebar from "../components/group-list-sidebar";
import PartnerDetailPanel from "../components/partner-detail-panel";
import RecipientTable from "../components/recipient-table";
import { listPartners, listPartnerMembers, getAppSettings } from "../lib/tauri-api";
import { useSettingsContext } from "../contexts/settings-context";
import type { Partner, PartnerMember } from "../types";

export default function PartnersPage() {
  const [groups, setGroups] = useState<Partner[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const selectedGroupIdRef = useRef<string | null>(null);
  // Keep ref in sync with state so callbacks always read the latest value
  useEffect(() => { selectedGroupIdRef.current = selectedGroupId; }, [selectedGroupId]);
  const [recipients, setRecipients] = useState<PartnerMember[]>([]);
  const [selectedMemberId, setSelectedMemberId] = useState<string | null>(null);
  const [showAddRecipient, setShowAddRecipient] = useState(false);
  const [loading, setLoading] = useState(false);
  const [desktopPath, setDesktopPath] = useState("");

  const { outputDataDir } = useSettingsContext();

  // Get Rust-resolved desktop path as fallback when outputDataDir is empty
  useEffect(() => {
    getAppSettings()
      .then((s) => setDesktopPath(s.output_data_dir || ""))
      .catch(() => {});
  }, []);

  useEffect(() => { loadGroups(); }, []);
  useEffect(() => {
    setSelectedMemberId(null);
    if (selectedGroupId) loadRecipients(selectedGroupId);
    else setRecipients([]);
  }, [selectedGroupId]);

  const loadGroups = async () => {
    try {
      const gs = await listPartners();
      setGroups(gs);
      // Read ref to avoid stale closure — ref is always current
      if (gs.length > 0 && !selectedGroupIdRef.current) setSelectedGroupId(gs[0].id);
    } catch { /* ignore */ }
  };

  const loadRecipients = async (groupId: string) => {
    setLoading(true);
    try {
      setRecipients(await listPartnerMembers(groupId));
    } catch {
      setRecipients([]);
    } finally {
      setLoading(false);
    }
  };

  const handleGroupRefresh = async () => {
    await loadGroups();
    if (selectedGroupId) await loadRecipients(selectedGroupId);
  };

  const selectedGroup = groups.find((g) => g.id === selectedGroupId) ?? null;
  const selectedMember = recipients.find((r) => r.id === selectedMemberId) ?? null;
  const effectiveBaseDir = outputDataDir || desktopPath;

  return (
    <div style={{ height: "100%", display: "flex", margin: "-24px" }}>
      <GroupListSidebar
        groups={groups}
        selectedId={selectedGroupId}
        onSelect={setSelectedGroupId}
        onRefresh={handleGroupRefresh}
      />

      <div style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column" }}>
        {selectedGroup ? (
          <>
            <div style={{ padding: "12px 16px", borderBottom: "1px solid var(--color-border-light)", background: "var(--color-bg-content)" }}>
              <h3 style={{ fontSize: "var(--font-size-lg)", color: "var(--color-text-on-light)" }}>
                {selectedGroup.name}
              </h3>
            </div>
            {loading ? (
              <div style={{ padding: 24, color: "var(--color-text-muted-light)" }}>Loading…</div>
            ) : (
              <RecipientTable
                recipients={recipients}
                onRefresh={() => loadRecipients(selectedGroupId!)}
                onAddRecipient={() => setShowAddRecipient(true)}
                onRowSelect={setSelectedMemberId}
                selectedMemberId={selectedMemberId}
                partnerName={selectedGroup.name}
                outputDataDir={effectiveBaseDir}
                desktopPath={desktopPath}
              />
            )}
          </>
        ) : (
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--color-text-muted-light)", fontSize: "var(--font-size-md)" }}>
            Select or create a partner to manage members
          </div>
        )}
      </div>

      <PartnerDetailPanel
        partner={selectedGroup}
        member={selectedMember}
        members={recipients}
      />

      {showAddRecipient && selectedGroupId && (
        <AddRecipientDialog
          partnerId={selectedGroupId}
          onAdded={() => {
            setShowAddRecipient(false);
            loadRecipients(selectedGroupId);
            loadGroups();
          }}
          onCancel={() => setShowAddRecipient(false)}
        />
      )}
    </div>
  );
}
