import { describe, expect, test } from "bun:test";

import { appRoutes } from "./routes";

describe("appRoutes", () => {
  test("maps the three admin pages and not found route", () => {
    const childPaths = appRoutes[0]?.children?.map(route => route.index ? "index" : route.path);

    expect(childPaths).toEqual(["index", "api-tester", "settings", "*"]);
  });
});
