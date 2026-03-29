import { useCallback, useEffect, useState } from "react";
import { getTokenStatus } from "../lib/tauri-api";
import type { TokenStatusResponse } from "../types";

const DEFAULT: TokenStatusResponse = {
  status: "disconnected",
  cert_cn: null,
  dll_found: false,
};

/// Polls get_token_status every intervalMs (default 20s).
/// Returns spread of TokenStatusResponse + refresh() for immediate update.
export function useTokenStatus(intervalMs = 20_000) {
  const [tokenState, setTokenState] = useState<TokenStatusResponse>(DEFAULT);

  const refresh = useCallback(async () => {
    try {
      setTokenState(await getTokenStatus());
    } catch {
      setTokenState(DEFAULT);
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, intervalMs);
    return () => clearInterval(id);
  }, [refresh, intervalMs]);

  return { ...tokenState, refresh };
}
