import { NavLink } from "react-router-dom";

import { type SettingsSectionId } from "@/components/app/account-shell";
import { SidebarAccountMenu } from "@/components/app/sidebar/sidebar-account-menu";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { navItems } from "@/router/nav-items";

export const APP_SIDEBAR_CLASS_NAME =
  "hidden border-border/70 md:flex md:h-screen md:w-64 md:border-r";
export const APP_SIDEBAR_PANEL_CLASS_NAME =
  "flex h-full w-full flex-col gap-6 bg-card/80 px-4 py-5 backdrop-blur";

type AppSidebarProps = {
  onOpenSettings: (section?: SettingsSectionId, errorMessage?: string | null) => void;
};

type SidebarNavigationProps = {
  onNavigate?: () => void;
};

export function SidebarNavigation({ onNavigate }: SidebarNavigationProps) {
  return (
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
          onClick={onNavigate}
        >
          <item.icon className="size-4" />
          <span className="font-medium">{item.title}</span>
        </NavLink>
      ))}
    </nav>
  );
}

export function AppSidebar({ onOpenSettings }: AppSidebarProps) {
  return (
    <aside className={APP_SIDEBAR_CLASS_NAME}>
      <div className={APP_SIDEBAR_PANEL_CLASS_NAME}>
        <div>
          <h2 className="text-lg font-semibold tracking-tight text-foreground">Irminsul Admin</h2>
        </div>

        <SidebarNavigation />

        <SidebarAccountMenu onOpenSettings={onOpenSettings} />
      </div>
    </aside>
  );
}
