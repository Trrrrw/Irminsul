import { describe, expect, test } from "bun:test";

import type { AdminUser } from "@/components/app/admin-session-context";

import {
  buildAccountFormState,
  resolveThemeAppearance,
  SETTINGS_DIALOG_CLASS_NAMES,
} from "./settings-dialog";

describe("settings dialog helpers", () => {
  test("builds account form state from the current admin user", () => {
    const user: AdminUser = {
      id: 1,
      username: "admin",
      email: "admin@example.com",
      role: "owner",
      status: "active",
      must_change_password: false,
      must_change_username: false,
      must_set_email: false,
    };

    expect(buildAccountFormState(user)).toEqual({
      username: "admin",
      email: "admin@example.com",
      currentPassword: "",
      newPassword: "",
    });
  });

  test("resolves the effective theme from mode and system preference", () => {
    expect(resolveThemeAppearance("light", true)).toBe("light");
    expect(resolveThemeAppearance("dark", false)).toBe("dark");
    expect(resolveThemeAppearance("system", true)).toBe("dark");
    expect(resolveThemeAppearance("system", false)).toBe("light");
  });

  test("keeps the dialog scrollable on narrow screens", () => {
    expect(SETTINGS_DIALOG_CLASS_NAMES.content).toContain("max-h-[calc(100dvh-1rem)]");
    expect(SETTINGS_DIALOG_CLASS_NAMES.content).toContain("overflow-y-auto");
    expect(SETTINGS_DIALOG_CLASS_NAMES.content).not.toContain("overflow-hidden");
    expect(SETTINGS_DIALOG_CLASS_NAMES.layout).toContain("min-h-0");
    expect(SETTINGS_DIALOG_CLASS_NAMES.layout).toContain("md:min-h-[36rem]");
  });

  test("stacks general setting rows on narrow screens so labels do not wrap", () => {
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingRow).toContain("flex-col");
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingRow).toContain("items-start");
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingRow).toContain("md:flex-row");
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingTitle).toContain("whitespace-nowrap");
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingControl).toContain("w-full");
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingControl).not.toContain("min-w-40");
    expect(SETTINGS_DIALOG_CLASS_NAMES.generalSettingControl).toContain("md:w-40");
  });
});
