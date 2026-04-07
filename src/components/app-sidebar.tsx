import { useState } from "react";
import { NavLink } from "react-router-dom";
import { Lock, Unlock, Users, FileKey, Settings, ChevronLeft, ChevronRight } from "lucide-react";

const NAV_ITEMS = [
  { to: "/encrypt",     label: "Encrypt",     Icon: Lock },
  { to: "/decrypt",     label: "Decrypt",     Icon: Unlock },
  { to: "/groups",      label: "Partners",    Icon: Users },
  { to: "/license-gen", label: "License Gen", Icon: FileKey },
  { to: "/settings",    label: "Settings",    Icon: Settings },
];

export default function AppSidebar() {
  const [collapsed, setCollapsed] = useState<boolean>(() =>
    localStorage.getItem("sidebar-collapsed") === "true"
  );

  const toggle = () => {
    setCollapsed((prev) => {
      localStorage.setItem("sidebar-collapsed", String(!prev));
      return !prev;
    });
  };

  const w = collapsed ? 56 : 200;

  return (
    <nav
      style={{
        width: w,
        minWidth: w,
        background: "var(--color-bg-sidebar)",
        borderRight: "1px solid var(--color-border-dark)",
        display: "flex",
        flexDirection: "column",
        transition: "width var(--transition-base)",
        overflow: "hidden",
      }}
    >
      {/* App name */}
      <div
        style={{
          padding: collapsed ? "20px 0 16px" : "20px 16px 16px",
          borderBottom: "1px solid var(--color-border-dark)",
          marginBottom: 8,
          textAlign: collapsed ? "center" : "left",
          flexShrink: 0,
        }}
      >
        {collapsed ? null : (
          <>
            <div
              style={{
                fontSize: "var(--font-size-lg)",
                fontWeight: "var(--font-weight-bold)",
                color: "var(--color-accent-primary)",
                whiteSpace: "nowrap",
              }}
            >
              CAHTQT
            </div>
            <div
              style={{
                fontSize: "var(--font-size-xs)",
                color: "var(--color-text-secondary)",
                marginTop: 2,
                whiteSpace: "nowrap",
              }}
            >
              PKI Encryption
            </div>
          </>
        )}
      </div>

      {/* Navigation links */}
      <div style={{ flex: 1 }}>
        {NAV_ITEMS.map(({ to, label, Icon }) => (
          <NavLink
            key={to}
            to={to}
            style={({ isActive }) => ({
              display: "flex",
              alignItems: "center",
              justifyContent: collapsed ? "center" : "flex-start",
              gap: collapsed ? 0 : 12,
              height: 40,
              padding: collapsed ? 0 : "0 16px",
              color: isActive ? "var(--color-text-primary)" : "var(--color-text-secondary)",
              background: isActive ? "var(--color-accent-primary)" : "transparent",
              textDecoration: "none",
              fontSize: "var(--font-size-base)",
              fontWeight: "var(--font-weight-medium)",
              borderRadius: "var(--radius-md)",
              margin: "2px 8px",
              transition: "background var(--transition-base), color var(--transition-base)",
            })}
            title={collapsed ? label : undefined}
          >
            {({ isActive }) => (
              <>
                <Icon
                  size={18}
                  color={isActive ? "var(--color-text-primary)" : "var(--color-text-secondary)"}
                />
                {!collapsed && <span style={{ whiteSpace: "nowrap" }}>{label}</span>}
              </>
            )}
          </NavLink>
        ))}
      </div>

      {/* Collapse toggle */}
      <button
        onClick={toggle}
        style={{
          height: 32,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "transparent",
          border: "none",
          borderTop: "1px solid var(--color-border-dark)",
          color: "var(--color-text-label)",
          cursor: "pointer",
          padding: 0,
          flexShrink: 0,
        }}
        title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
      >
        {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
      </button>
    </nav>
  );
}
