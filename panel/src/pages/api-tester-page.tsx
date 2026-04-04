import { useEffect, useRef, useState, type FocusEvent, type FormEvent } from "react";

import { PageHeader } from "@/components/app/page-header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

const HTTP_METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] as const;
const QUERY_PARAMETER_METHODS = new Set<HttpMethod>(["GET", "HEAD", "OPTIONS"]);
const JSON_REQUEST_BODY_METHODS = new Set<HttpMethod>(["POST", "PUT", "PATCH", "DELETE"]);

type HttpMethod = (typeof HTTP_METHODS)[number];

export type ApiRouteDescriptor = {
  method: string;
  path: string;
};

type QueryParameterField = {
  id: string;
  key: string;
  value: string;
};

type ApiEndpointOption = {
  path: string;
  methods: string[];
};

let nextQueryParameterFieldId = 0;

function createEmptyQueryParameterField(): QueryParameterField {
  nextQueryParameterFieldId += 1;

  return {
    id: `query-parameter-${nextQueryParameterFieldId}`,
    key: "",
    value: "",
  };
}

export function buildApiEndpointOptions(routes: ApiRouteDescriptor[]): ApiEndpointOption[] {
  const optionsByPath = new Map<string, Set<string>>();

  for (const route of routes) {
    const option = optionsByPath.get(route.path) ?? new Set<string>();
    option.add(route.method.toUpperCase());
    optionsByPath.set(route.path, option);
  }

  return [...optionsByPath.entries()]
    .sort(([leftPath], [rightPath]) => leftPath.localeCompare(rightPath))
    .map(([path, methods]) => ({
      path,
      methods: [...methods].sort((leftMethod, rightMethod) => {
        return HTTP_METHODS.indexOf(leftMethod as HttpMethod) - HTTP_METHODS.indexOf(rightMethod as HttpMethod);
      }),
    }));
}

export function filterApiEndpointOptions(options: ApiEndpointOption[], query: string) {
  const normalizedQuery = query.trim().toLowerCase();

  if (!normalizedQuery) {
    return options;
  }

  return options.filter(option => option.path.toLowerCase().includes(normalizedQuery));
}

export function resolveMethodForEndpointSelection(currentMethod: string, methods: string[]) {
  if (!methods.length || methods.includes(currentMethod)) {
    return currentMethod;
  }

  return methods[0]!;
}

export function buildRequestUrl(endpoint: string, queryParameters: Array<Pick<QueryParameterField, "key" | "value">>, baseUrl: string) {
  const url = new URL(endpoint, baseUrl);

  for (const queryParameter of queryParameters) {
    const normalizedKey = queryParameter.key.trim();

    if (!normalizedKey) {
      continue;
    }

    url.searchParams.set(normalizedKey, queryParameter.value);
  }

  return url.toString();
}

export function parseJsonRequestBody(bodyText: string) {
  const normalizedBodyText = bodyText.trim();

  if (!normalizedBodyText) {
    return {
      body: null,
      error: null,
    };
  }

  try {
    return {
      body: JSON.stringify(JSON.parse(normalizedBodyText)),
      error: null,
    };
  } catch {
    return {
      body: null,
      error: "JSON 请求体格式不正确。",
    };
  }
}

async function readResponseBody(response: Response) {
  const responseText = await response.text();

  if (!responseText.trim()) {
    return `${response.status} ${response.statusText}`;
  }

  try {
    return `${response.status} ${response.statusText}\n\n${JSON.stringify(JSON.parse(responseText), null, 2)}`;
  } catch {
    return `${response.status} ${response.statusText}\n\n${responseText}`;
  }
}

export function ApiTesterPage() {
  const responseInputRef = useRef<HTMLTextAreaElement>(null);
  const [method, setMethod] = useState<string>("GET");
  const [endpoint, setEndpoint] = useState("/api/routes");
  const [routeCatalog, setRouteCatalog] = useState<ApiRouteDescriptor[]>([]);
  const [routeCatalogError, setRouteCatalogError] = useState<string | null>(null);
  const [isLoadingRouteCatalog, setIsLoadingRouteCatalog] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isEndpointMenuOpen, setIsEndpointMenuOpen] = useState(false);
  const [queryParameters, setQueryParameters] = useState<QueryParameterField[]>([createEmptyQueryParameterField()]);
  const [jsonRequestBody, setJsonRequestBody] = useState("{\n  \n}");

  const endpointOptions = buildApiEndpointOptions(routeCatalog);
  const matchingEndpointOptions = filterApiEndpointOptions(endpointOptions, endpoint);
  const supportsQueryParameters = QUERY_PARAMETER_METHODS.has(method as HttpMethod);
  const supportsJsonRequestBody = JSON_REQUEST_BODY_METHODS.has(method as HttpMethod);

  useEffect(() => {
    let isActive = true;

    const loadRouteCatalog = async () => {
      try {
        const response = await fetch("/api/routes", {
          credentials: "include",
        });

        if (!response.ok) {
          throw new Error("接口目录加载失败。");
        }

        const payload = (await response.json()) as ApiRouteDescriptor[];

        if (!isActive) {
          return;
        }

        setRouteCatalog(payload);
        setRouteCatalogError(null);
      } catch (error) {
        if (!isActive) {
          return;
        }

        setRouteCatalogError(error instanceof Error ? error.message : "接口目录加载失败。");
      } finally {
        if (isActive) {
          setIsLoadingRouteCatalog(false);
        }
      }
    };

    void loadRouteCatalog();

    return () => {
      isActive = false;
    };
  }, []);

  const handleEndpointFieldBlur = (event: FocusEvent<HTMLDivElement>) => {
    if (!event.currentTarget.contains(event.relatedTarget)) {
      setIsEndpointMenuOpen(false);
    }
  };

  const handleEndpointSelection = (option: ApiEndpointOption) => {
    setEndpoint(option.path);
    setMethod(currentMethod => resolveMethodForEndpointSelection(currentMethod, option.methods));
    setIsEndpointMenuOpen(false);
  };

  const handleQueryParameterChange = (
    fieldId: string,
    fieldName: "key" | "value",
    nextValue: string,
  ) => {
    setQueryParameters(currentFields =>
      currentFields.map(field => {
        if (field.id !== fieldId) {
          return field;
        }

        return {
          ...field,
          [fieldName]: nextValue,
        };
      }),
    );
  };

  const handleAddQueryParameter = () => {
    setQueryParameters(currentFields => [...currentFields, createEmptyQueryParameterField()]);
  };

  const handleRemoveQueryParameter = (fieldId: string) => {
    setQueryParameters(currentFields => {
      const nextFields = currentFields.filter(field => field.id !== fieldId);
      return nextFields.length > 0 ? nextFields : [createEmptyQueryParameterField()];
    });
  };

  const testEndpoint = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    if (isSubmitting) {
      return;
    }

    const trimmedEndpoint = endpoint.trim();

    if (!trimmedEndpoint) {
      responseInputRef.current!.value = "请输入接口地址。";
      return;
    }

    setIsSubmitting(true);

    try {
      const url = buildRequestUrl(
        trimmedEndpoint,
        supportsQueryParameters ? queryParameters : [],
        location.href,
      );
      const jsonRequestBodyResult = supportsJsonRequestBody
        ? parseJsonRequestBody(jsonRequestBody)
        : { body: null, error: null };

      if (jsonRequestBodyResult.error) {
        responseInputRef.current!.value = jsonRequestBodyResult.error;
        return;
      }

      const headers = new Headers();
      if (jsonRequestBodyResult.body) {
        headers.set("Content-Type", "application/json");
      }

      const response = await fetch(url, {
        method,
        credentials: "include",
        headers,
        body: jsonRequestBodyResult.body ?? undefined,
      });

      responseInputRef.current!.value = await readResponseBody(response);
    } catch (error) {
      responseInputRef.current!.value = String(error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col gap-8">
      <PageHeader title="接口测试" />

      <Card>
        <CardHeader>
          <CardTitle>快速请求接口</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          <form onSubmit={testEndpoint} className="flex flex-col gap-3 md:flex-row md:items-start">
            <div className="w-full md:w-[140px]">
              <Label htmlFor="method" className="sr-only">
                Method
              </Label>
              <Select name="method" value={method} onValueChange={setMethod}>
                <SelectTrigger className="w-full" id="method">
                  <SelectValue placeholder="Method" />
                </SelectTrigger>
                <SelectContent align="start">
                  {HTTP_METHODS.map(methodOption => (
                    <SelectItem key={methodOption} value={methodOption}>
                      {methodOption}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="relative flex-1" onBlur={handleEndpointFieldBlur}>
              <Label htmlFor="endpoint" className="sr-only">
                Endpoint
              </Label>
              <Input
                id="endpoint"
                type="text"
                name="endpoint"
                value={endpoint}
                placeholder="/api/routes"
                className="w-full"
                autoComplete="off"
                spellCheck={false}
                onFocus={() => setIsEndpointMenuOpen(true)}
                onChange={event => {
                  setEndpoint(event.target.value);
                  setIsEndpointMenuOpen(true);
                }}
              />

              {isEndpointMenuOpen ? (
                <div className="absolute top-full z-20 mt-2 w-full overflow-hidden rounded-2xl border border-border/80 bg-popover/98 shadow-2xl backdrop-blur">
                  {matchingEndpointOptions.length > 0 ? (
                    <div className="max-h-72 overflow-y-auto p-1.5">
                      {matchingEndpointOptions.map(option => (
                        <button
                          key={option.path}
                          type="button"
                          className="flex w-full items-center justify-between gap-3 rounded-xl px-3 py-2 text-left text-sm transition hover:bg-accent hover:text-accent-foreground"
                          onClick={() => handleEndpointSelection(option)}
                        >
                          <span className="truncate font-mono text-xs md:text-sm">{option.path}</span>
                          <span className="shrink-0 text-[11px] font-medium text-muted-foreground">
                            {option.methods.join(" ")}
                          </span>
                        </button>
                      ))}
                    </div>
                  ) : (
                    <div className="px-3 py-2 text-sm text-muted-foreground">未匹配到接口</div>
                  )}
                </div>
              ) : null}
            </div>

            <Button type="submit" variant="secondary" disabled={isSubmitting}>
              {isSubmitting ? "Sending..." : "Send"}
            </Button>
          </form>

          <div className="text-sm text-muted-foreground">
            {routeCatalogError
              ? routeCatalogError
              : isLoadingRouteCatalog
                ? "正在加载业务接口目录..."
                : `已加载 ${endpointOptions.length} 个业务接口路径，可输入筛选或直接选择。`}
          </div>

          {supportsQueryParameters ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between gap-3">
                <Label>查询参数</Label>
                <Button type="button" variant="outline" size="sm" onClick={handleAddQueryParameter}>
                  添加参数
                </Button>
              </div>

              <div className="space-y-2">
                {queryParameters.map(queryParameter => (
                  <div key={queryParameter.id} className="flex flex-col gap-2 md:flex-row">
                    <Input
                      type="text"
                      value={queryParameter.key}
                      placeholder="参数名"
                      className="md:flex-1"
                      onChange={event => handleQueryParameterChange(queryParameter.id, "key", event.target.value)}
                    />
                    <Input
                      type="text"
                      value={queryParameter.value}
                      placeholder="参数值"
                      className="md:flex-1"
                      onChange={event => handleQueryParameterChange(queryParameter.id, "value", event.target.value)}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="md:self-center"
                      onClick={() => handleRemoveQueryParameter(queryParameter.id)}
                    >
                      删除
                    </Button>
                  </div>
                ))}
              </div>

              <div className="text-xs text-muted-foreground">
                会将非空参数名按键值对拼接到请求 URL 上喵～
              </div>
            </div>
          ) : null}

          {supportsJsonRequestBody ? (
            <div className="space-y-2">
              <Label htmlFor="request-body">JSON 请求体</Label>
              <Textarea
                id="request-body"
                value={jsonRequestBody}
                placeholder='{\n  "name": "Irminsul"\n}'
                className="min-h-[220px] resize-y font-mono text-xs"
                onChange={event => setJsonRequestBody(event.target.value)}
              />
              <div className="text-xs text-muted-foreground">发送前会校验 JSON 格式，并以 `application/json` 提交。</div>
            </div>
          ) : null}

          <div className="space-y-2">
            <Label htmlFor="response">Response</Label>
            <Textarea
              ref={responseInputRef}
              id="response"
              readOnly
              placeholder="Response will appear here..."
              className="min-h-[320px] resize-y font-mono text-xs"
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
