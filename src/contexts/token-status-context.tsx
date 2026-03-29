import { createContext, useContext, ReactNode } from "react";
import { useTokenStatus } from "../hooks/use-token-status";

// Infer return type from the hook to avoid manual duplication
type TokenStatusContextType = ReturnType<typeof useTokenStatus>;

const TokenStatusContext = createContext<TokenStatusContextType | null>(null);

export function TokenStatusProvider({ children }: { children: ReactNode }) {
  const value = useTokenStatus();
  return (
    <TokenStatusContext.Provider value={value}>
      {children}
    </TokenStatusContext.Provider>
  );
}

export function useTokenStatusContext(): TokenStatusContextType {
  const ctx = useContext(TokenStatusContext);
  if (!ctx) throw new Error("useTokenStatusContext must be used within TokenStatusProvider");
  return ctx;
}
