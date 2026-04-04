import { useEffect, useRef, useState } from "react";
import { LogOut, MoreHorizontal, Settings2 } from "lucide-react";

import {
  accountMenuItems,
  getAccountInitials,
  type AccountMenuItemId,
  type SettingsSectionId,
} from "@/components/app/account-shell";
import { useAdminSession } from "@/components/app/admin-session-context";
import { cn } from "@/lib/utils";

type SidebarAccountMenuProps = {
  onOpenSettings: (section?: SettingsSectionId, errorMessage?: string | null) => void;
};

type MobileAccountMenuProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onToggle: () => void;
  onOpenSettings: (section?: SettingsSectionId, errorMessage?: string | null) => void;
};

export function getAccountMenuIcon(menuId: AccountMenuItemId) {
  switch (menuId) {
    case "settings":
      return "settings";
    case "logout":
      return "logout";
    default:
      return "settings";
  }
}

export function isDangerAccountMenuItem(item: (typeof accountMenuItems)[number]) {
  return item.tone === "danger";
}

export function getAccountMenuButtonLabel(username: string) {
  return `打开${username}的账户菜单`;
}

async function runAccountMenuAction(
  actionId: AccountMenuItemId,
  logout: () => Promise<void>,
  onOpenSettings: (section?: SettingsSectionId, errorMessage?: string | null) => void,
) {
  if (actionId === "settings") {
    onOpenSettings("general", null);
    return;
  }

  if (actionId === "logout") {
    try {
      await logout();
    } catch (error) {
      onOpenSettings("account", error instanceof Error ? error.message : "退出登录失败，请稍后重试。");
    }
  }
}

type AccountMenuItemsPanelProps = {
  className: string;
  onSelect: (actionId: AccountMenuItemId) => void;
};

function AccountMenuItemsPanel({ className, onSelect }: AccountMenuItemsPanelProps) {
  return (
    <div className={className}>
      {accountMenuItems.map(item => (
        <button
          key={item.id}
          type="button"
          className={cn(
            "flex w-full items-center gap-2 rounded-xl px-3 py-2.5 text-left text-sm font-medium transition-colors",
            isDangerAccountMenuItem(item)
              ? "text-destructive hover:bg-destructive/10"
              : "text-foreground hover:bg-accent",
          )}
          onClick={() => onSelect(item.id)}
        >
          {getAccountMenuIcon(item.id) === "settings" ? <Settings2 className="size-4" /> : <LogOut className="size-4" />}
          <span>{item.label}</span>
        </button>
      ))}
    </div>
  );
}

export function SidebarAccountMenu({ onOpenSettings }: SidebarAccountMenuProps) {
  const { logout, user } = useAdminSession();
  const accountMenuRef = useRef<HTMLDivElement>(null);
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    function handlePointerDown(event: PointerEvent) {
      if (!accountMenuRef.current?.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    }

    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen]);

  async function handleMenuAction(actionId: AccountMenuItemId) {
    setIsOpen(false);
    await runAccountMenuAction(actionId, logout, onOpenSettings);
  }

  return (
    <div ref={accountMenuRef} className="relative mt-auto">
      {isOpen ? (
        <AccountMenuItemsPanel
          className="absolute right-0 bottom-full left-0 z-20 mb-2 overflow-hidden rounded-2xl border border-border/80 bg-popover/98 p-1.5 text-popover-foreground shadow-2xl backdrop-blur"
          onSelect={actionId => void handleMenuAction(actionId)}
        />
      ) : null}

      <button
        type="button"
        className="flex w-full items-center gap-2.5 rounded-full border border-border/80 bg-background/80 px-2.5 py-2 text-left shadow-sm transition-colors hover:border-border hover:bg-accent/40"
        aria-label={getAccountMenuButtonLabel(user.username)}
        onClick={() => setIsOpen(current => !current)}
      >
        <div className="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary">
          {getAccountInitials(user.username)}
        </div>

        <div className="min-w-0 flex-1 truncate text-sm font-semibold text-foreground">
          {user.username}
        </div>

        <div className="flex size-7 shrink-0 items-center justify-center rounded-full bg-secondary/80 text-muted-foreground">
          <MoreHorizontal className="size-3.5" />
        </div>
      </button>
    </div>
  );
}

export function MobileAccountMenu({ open, onOpenChange, onToggle, onOpenSettings }: MobileAccountMenuProps) {
  const { logout, user } = useAdminSession();

  async function handleMenuAction(actionId: AccountMenuItemId) {
    onOpenChange(false);
    await runAccountMenuAction(actionId, logout, onOpenSettings);
  }

  return (
    <div className="relative md:hidden">
      <button
        type="button"
        className="flex size-9 items-center justify-center rounded-full border border-border/80 bg-background/80 text-sm font-semibold text-primary shadow-sm transition-colors hover:border-border hover:bg-accent/40"
        aria-label={getAccountMenuButtonLabel(user.username)}
        onClick={onToggle}
      >
        {getAccountInitials(user.username)}
      </button>

      {open ? (
        <>
          <button
            type="button"
            className="fixed inset-0 z-30 md:hidden"
            aria-label="关闭账户菜单"
            onClick={() => onOpenChange(false)}
          />

          <AccountMenuItemsPanel
            className="absolute top-full right-0 z-40 mt-2 w-48 overflow-hidden rounded-2xl border border-border/80 bg-popover/98 p-1.5 text-popover-foreground shadow-2xl backdrop-blur"
            onSelect={actionId => void handleMenuAction(actionId)}
          />
        </>
      ) : null}
    </div>
  );
}
