import { useEffect, useState } from "react";
import { Menu } from "lucide-react";
import { Outlet, useLocation } from "react-router-dom";

import { type SettingsSectionId } from "@/components/app/account-shell";
import { MobileAccountMenu } from "@/components/app/sidebar/sidebar-account-menu";
import { AppSidebar, APP_SIDEBAR_PANEL_CLASS_NAME, SidebarNavigation } from "@/components/app/sidebar/app-sidebar";
import { SettingsDialog } from "@/components/app/settings/settings-dialog";
import { Button } from "@/components/ui/button";

export const DASHBOARD_LAYOUT_CLASS_NAMES = {
  root: "h-screen overflow-hidden bg-background text-foreground",
  grid: "relative h-screen md:grid md:grid-cols-[18rem_minmax(0,1fr)]",
  main: "h-screen overflow-y-auto bg-background/95",
  mobileHeader:
    "sticky top-0 z-20 flex h-16 items-center justify-between gap-3 border-b border-border/70 bg-background/95 px-4 backdrop-blur md:hidden",
  content: "mx-auto flex w-full max-w-6xl flex-col gap-8 px-4 py-6 sm:px-6 lg:px-10 lg:py-10",
} as const;

export function DashboardLayout() {
  const location = useLocation();
  const [isMobileNavOpen, setIsMobileNavOpen] = useState(false);
  const [isMobileAccountMenuOpen, setIsMobileAccountMenuOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settingsOpenSection, setSettingsOpenSection] = useState<SettingsSectionId>("general");
  const [settingsAccountError, setSettingsAccountError] = useState<string | null>(null);

  useEffect(() => {
    setIsMobileNavOpen(false);
    setIsMobileAccountMenuOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    if (!isMobileNavOpen && !isMobileAccountMenuOpen) {
      return;
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setIsMobileNavOpen(false);
        setIsMobileAccountMenuOpen(false);
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isMobileAccountMenuOpen, isMobileNavOpen]);

  function handleOpenSettings(section: SettingsSectionId = "general", errorMessage: string | null = null) {
    setSettingsOpenSection(section);
    setSettingsAccountError(errorMessage);
    setIsSettingsOpen(true);
  }

  function handleToggleMobileNav() {
    setIsMobileAccountMenuOpen(false);
    setIsMobileNavOpen(current => !current);
  }

  function handleToggleMobileAccountMenu() {
    setIsMobileNavOpen(false);
    setIsMobileAccountMenuOpen(current => !current);
  }

  function handleCloseOverlays() {
    setIsMobileNavOpen(false);
    setIsMobileAccountMenuOpen(false);
  }

  return (
    <div className={DASHBOARD_LAYOUT_CLASS_NAMES.root}>
      <div className={DASHBOARD_LAYOUT_CLASS_NAMES.grid}>
        <AppSidebar onOpenSettings={handleOpenSettings} />
        <main className={DASHBOARD_LAYOUT_CLASS_NAMES.main}>
          <header className={DASHBOARD_LAYOUT_CLASS_NAMES.mobileHeader}>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              className="rounded-xl"
              aria-label={isMobileNavOpen ? "关闭导航菜单" : "打开导航菜单"}
              onClick={handleToggleMobileNav}
            >
              <Menu className="size-5" />
            </Button>

            <div className="min-w-0 flex-1 text-center text-sm font-semibold tracking-tight text-foreground">
              Irminsul Admin
            </div>

            <MobileAccountMenu
              open={isMobileAccountMenuOpen}
              onOpenChange={setIsMobileAccountMenuOpen}
              onToggle={handleToggleMobileAccountMenu}
              onOpenSettings={handleOpenSettings}
            />
          </header>

          <div className={DASHBOARD_LAYOUT_CLASS_NAMES.content}>
            <Outlet />
          </div>
        </main>
      </div>

      {isMobileNavOpen ? (
        <>
          <button
            type="button"
            className="fixed inset-0 z-30 bg-black/45 backdrop-blur-sm md:hidden"
            aria-label="关闭导航菜单"
            onClick={handleCloseOverlays}
          />

          <aside className="fixed inset-y-0 left-0 z-40 w-full max-w-64 border-r border-border/70 bg-card/95 shadow-2xl md:hidden">
            <div className={APP_SIDEBAR_PANEL_CLASS_NAME}>
              <div>
                <h2 className="text-lg font-semibold tracking-tight text-foreground">Irminsul Admin</h2>
              </div>

              <SidebarNavigation onNavigate={handleCloseOverlays} />
            </div>
          </aside>
        </>
      ) : null}

      <SettingsDialog
        open={isSettingsOpen}
        onOpenChange={setIsSettingsOpen}
        initialSection={settingsOpenSection}
        initialAccountError={settingsAccountError}
      />
    </div>
  );
}
