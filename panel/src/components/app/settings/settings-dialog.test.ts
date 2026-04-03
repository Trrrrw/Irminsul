import { describe, expect, test } from "bun:test";

import type { AdminUser } from "@/components/app/admin-session-context";

import { buildAccountFormState, resolveThemeAppearance } from "./settings-dialog";

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
});
