import { describe, expect, test } from "bun:test";

import {
  accountMenuItems,
  generalSettingsItems,
  settingsSections,
  themeOptions,
  getAccountBadgeLabel,
  getAccountInitials,
} from "./account-shell";

describe("account shell config", () => {
  test("keeps the account menu focused on settings and logout", () => {
    expect(accountMenuItems).toEqual([
      { id: "settings", label: "系统设置" },
      { id: "logout", label: "退出登录", tone: "danger" },
    ]);
  });

  test("exposes the confirmed settings sections and general options", () => {
    expect(settingsSections).toEqual([
      { id: "general", label: "通用设置" },
      { id: "account", label: "账号管理" },
    ]);

    expect(generalSettingsItems).toEqual([
      { id: "language", label: "语言" },
      { id: "theme", label: "主题" },
    ]);
  });

  test("supports the three theme modes", () => {
    expect(themeOptions).toEqual([
      { id: "light", label: "浅色" },
      { id: "dark", label: "深色" },
      { id: "system", label: "跟随系统" },
    ]);
  });

  test("derives the sidebar badge copy from role and username", () => {
    expect(getAccountBadgeLabel("owner")).toBe("所有者");
    expect(getAccountBadgeLabel("viewer")).toBe("管理员");
    expect(getAccountInitials("admin-user")).toBe("AD");
    expect(getAccountInitials("浮浮酱")).toBe("浮");
  });
});
