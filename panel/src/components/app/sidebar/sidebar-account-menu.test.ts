import { describe, expect, test } from "bun:test";

import {
  getAccountMenuButtonLabel,
  getAccountMenuIcon,
  isDangerAccountMenuItem,
} from "./sidebar-account-menu";

describe("sidebar account menu helpers", () => {
  test("maps menu ids to the expected icon names", () => {
    expect(getAccountMenuIcon("settings")).toBe("settings");
    expect(getAccountMenuIcon("logout")).toBe("logout");
  });

  test("recognizes the destructive logout action", () => {
    expect(isDangerAccountMenuItem({ id: "settings", label: "系统设置" })).toBe(false);
    expect(isDangerAccountMenuItem({ id: "logout", label: "退出登录", tone: "danger" })).toBe(true);
  });

  test("builds an accessible compact trigger label for the mobile header", () => {
    expect(getAccountMenuButtonLabel("阿树")).toBe("打开阿树的账户菜单");
  });
});
