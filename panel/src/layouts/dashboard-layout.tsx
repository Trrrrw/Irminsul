import { Outlet } from "react-router-dom";

import { AppSidebar } from "@/components/app/app-sidebar";

export function DashboardLayout() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="relative min-h-screen md:grid md:grid-cols-[18rem_minmax(0,1fr)]">
        <AppSidebar />
        <main className="min-h-screen bg-background/95">
          <div className="mx-auto flex min-h-screen w-full max-w-6xl flex-col gap-8 px-4 py-6 sm:px-6 lg:px-10 lg:py-10">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
