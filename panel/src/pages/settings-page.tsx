import { PageHeader } from "@/components/app/page-header";

export function SettingsPage() {
  return (
    <div className="flex flex-col gap-8">
      <PageHeader title="系统设置" />
      <div>
        <span className="inline-flex items-center rounded-full border border-amber-500/30 bg-amber-500/10 px-2.5 py-1 text-xs font-medium tracking-wide text-amber-700">
          Pending
        </span>
      </div>
    </div>
  );
}
