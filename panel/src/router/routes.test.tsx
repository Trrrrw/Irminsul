import { describe, expect, test } from "bun:test";

import { appRoutes } from "./routes";

describe("appRoutes", () => {
  test("maps the dashboard pages without a standalone settings route", () => {
    const dashboardChildren = appRoutes[2]?.children?.[0]?.children?.map(route => route.index ? "index" : route.path);

    expect(dashboardChildren).toEqual(["index", "api-tester", "*"]);
  });
});
