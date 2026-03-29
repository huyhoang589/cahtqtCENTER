import { createContext, useContext, ReactNode } from "react";
import { useSettingsStore } from "../hooks/use-settings-store";

// Infer return type from the hook to avoid manual duplication
type SettingsContextType = ReturnType<typeof useSettingsStore>;

const SettingsContext = createContext<SettingsContextType | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const value = useSettingsStore();
  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettingsContext(): SettingsContextType {
  const ctx = useContext(SettingsContext);
  if (!ctx) throw new Error("useSettingsContext must be used within SettingsProvider");
  return ctx;
}
