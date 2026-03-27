import { describe, expect, test } from "bun:test";

import { navItems } from "./nav-items";

describe("navItems", () => {
  test("exposes the expected admin navigation order", () => {
    expect(navItems.map(item => item.title)).toEqual(["概览", "接口测试", "系统设置"]);
    expect(navItems.map(item => item.to)).toEqual(["/", "/api-tester", "/settings"]);
  });
});
