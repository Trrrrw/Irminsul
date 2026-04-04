import { describe, expect, test } from "bun:test";

import {
  buildApiEndpointOptions,
  buildRequestUrl,
  filterApiEndpointOptions,
  parseJsonRequestBody,
  resolveMethodForEndpointSelection,
  type ApiRouteDescriptor,
} from "./api-tester-page";

describe("api tester page helpers", () => {
  test("builds unique endpoint options grouped by path", () => {
    const routes: ApiRouteDescriptor[] = [
      { method: "POST", path: "/api/admin/auth/login" },
      { method: "GET", path: "/api/routes" },
      { method: "GET", path: "/api/admin/auth/login" },
      { method: "POST", path: "/api/admin/auth/login" },
    ];

    expect(buildApiEndpointOptions(routes)).toEqual([
      { path: "/api/admin/auth/login", methods: ["GET", "POST"] },
      { path: "/api/routes", methods: ["GET"] },
    ]);
  });

  test("filters endpoint options by path text only", () => {
    const options = [
      { path: "/api/admin/auth/login", methods: ["GET", "POST"] },
      { path: "/api/admin/users/me", methods: ["PATCH"] },
      { path: "/mcp", methods: ["POST"] },
    ];

    expect(filterApiEndpointOptions(options, "auth")).toEqual([
      { path: "/api/admin/auth/login", methods: ["GET", "POST"] },
    ]);
    expect(filterApiEndpointOptions(options, "/API/ADMIN")).toEqual([
      { path: "/api/admin/auth/login", methods: ["GET", "POST"] },
      { path: "/api/admin/users/me", methods: ["PATCH"] },
    ]);
  });

  test("keeps supported method or falls back to the first available method", () => {
    expect(resolveMethodForEndpointSelection("POST", ["GET", "POST"])).toBe("POST");
    expect(resolveMethodForEndpointSelection("DELETE", ["GET", "POST"])).toBe("GET");
    expect(resolveMethodForEndpointSelection("DELETE", [])).toBe("DELETE");
  });

  test("builds request urls by merging query pairs into the endpoint", () => {
    expect(
      buildRequestUrl(
        "/api/admin/users?locale=zh-CN&status=inactive",
        [
          { key: "status", value: "active" },
          { key: "page", value: "2" },
          { key: " ", value: "ignored" },
        ],
        "http://localhost:7040/admin",
      ),
    ).toBe("http://localhost:7040/api/admin/users?locale=zh-CN&status=active&page=2");
  });

  test("parses json request body and reports invalid input", () => {
    expect(parseJsonRequestBody("  ")).toEqual({
      body: null,
      error: null,
    });
    expect(parseJsonRequestBody('{ "name": "Irminsul", "enabled": true }')).toEqual({
      body: JSON.stringify({ name: "Irminsul", enabled: true }),
      error: null,
    });
    expect(parseJsonRequestBody("{ invalid json }")).toEqual({
      body: null,
      error: "JSON 请求体格式不正确。",
    });
  });
});
