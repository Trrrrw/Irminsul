import { useState } from "react";
import { NavLink } from "react-router-dom";

import { type SettingsSectionId } from "@/components/app/account-shell";
import { SidebarAccountMenu } from "@/components/app/sidebar/sidebar-account-menu";
import { SettingsDialog } from "@/components/app/settings/settings-dialog";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { navItems } from "@/router/nav-items";

export function AppSidebar() {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settingsOpenSection, setSettingsOpenSection] = useState<SettingsSectionId>("general");
  const [settingsAccountError, setSettingsAccountError] = useState<string | null>(null);

  function handleOpenSettings(section: SettingsSectionId = "general", errorMessage: string | null = null) {
    setSettingsOpenSection(section);
    setSettingsAccountError(errorMessage);
    setIsSettingsOpen(true);
  }

  return (
    <>
      <aside className="flex w-full max-w-64 flex-col gap-6 border-b border-border/70 bg-card/80 px-4 py-5 backdrop-blur md:min-h-screen md:w-64 md:border-r md:border-b-0">
        <div>
          <h2 className="text-lg font-semibold tracking-tight text-foreground">Irminsul Admin</h2>
        </div>

        <nav className="flex flex-1 flex-col gap-1.5">
          {navItems.map(item => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              className={({ isActive }) =>
                cn(
                  buttonVariants({ variant: "ghost", size: "sm" }),
                  "h-8 justify-start rounded-lg px-2.5 text-left text-sm font-medium transition-colors",
                  isActive
                    ? "bg-primary/10 text-primary hover:bg-primary/10 hover:text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground",
                )
              }
            >
              <item.icon className="size-4" />
              <span className="font-medium">{item.title}</span>
            </NavLink>
          ))}
        </nav>

        <SidebarAccountMenu onOpenSettings={handleOpenSettings} />
      </aside>

      <SettingsDialog
        open={isSettingsOpen}
        onOpenChange={setIsSettingsOpen}
        initialSection={settingsOpenSection}
        initialAccountError={settingsAccountError}
      />
    </>
  );
}
