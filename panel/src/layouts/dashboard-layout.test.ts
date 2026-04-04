import { describe, expect, test } from "bun:test";

import { APP_SIDEBAR_CLASS_NAME } from "@/components/app/sidebar/app-sidebar";

import { DASHBOARD_LAYOUT_CLASS_NAMES } from "./dashboard-layout";

describe("dashboard layout", () => {
  test("keeps the shell fixed to the viewport and scrolls inside the main pane", () => {
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.root).toContain("h-screen");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.root).toContain("overflow-hidden");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.grid).toContain("h-screen");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.main).toContain("h-screen");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.main).toContain("overflow-y-auto");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.content).not.toContain("min-h-screen");
  });

  test("keeps the sidebar pinned to the viewport height", () => {
    expect(APP_SIDEBAR_CLASS_NAME).toContain("hidden");
    expect(APP_SIDEBAR_CLASS_NAME).toContain("md:flex");
    expect(APP_SIDEBAR_CLASS_NAME).toContain("md:h-screen");
    expect(APP_SIDEBAR_CLASS_NAME).not.toContain("md:min-h-screen");
  });

  test("defines a mobile sticky header that only appears below the md breakpoint", () => {
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.mobileHeader).toContain("sticky");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.mobileHeader).toContain("top-0");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.mobileHeader).toContain("md:hidden");
    expect(DASHBOARD_LAYOUT_CLASS_NAMES.mobileHeader).toContain("backdrop-blur");
  });
});
