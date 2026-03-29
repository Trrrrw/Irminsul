import { Outlet } from "react-router-dom";

export function AuthLayout() {
  return (
    <div className="relative min-h-screen overflow-hidden bg-background text-foreground">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 top-0 h-64 bg-[radial-gradient(circle_at_top,theme(colors.primary/.16),transparent_60%)]"
      />
      <div
        aria-hidden
        className="pointer-events-none absolute inset-y-24 right-[-8rem] hidden size-80 rounded-full border border-border/60 bg-card/70 blur-3xl lg:block"
      />

      <div className="relative mx-auto flex min-h-screen w-full max-w-7xl items-center justify-center px-6 py-10 lg:px-10">
        <div className="w-full max-w-xl">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
