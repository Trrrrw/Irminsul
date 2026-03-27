import { useRef, type FormEvent } from "react";

import { PageHeader } from "@/components/app/page-header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

export function ApiTesterPage() {
  const responseInputRef = useRef<HTMLTextAreaElement>(null);

  const testEndpoint = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    try {
      const form = e.currentTarget;
      const formData = new FormData(form);
      const endpoint = formData.get("endpoint") as string;
      const url = new URL(endpoint, location.href);
      const method = formData.get("method") as string;
      const res = await fetch(url, { method });

      const data = await res.json();
      responseInputRef.current!.value = JSON.stringify(data, null, 2);
    } catch (error) {
      responseInputRef.current!.value = String(error);
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
          <form onSubmit={testEndpoint} className="flex flex-col gap-3 md:flex-row md:items-center">
            <div className="w-full md:w-[140px]">
              <Label htmlFor="method" className="sr-only">
                Method
              </Label>
              <Select name="method" defaultValue="GET">
                <SelectTrigger className="w-full" id="method">
                  <SelectValue placeholder="Method" />
                </SelectTrigger>
                <SelectContent align="start">
                  <SelectItem value="GET">GET</SelectItem>
                  <SelectItem value="PUT">PUT</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex-1">
              <Label htmlFor="endpoint" className="sr-only">
                Endpoint
              </Label>
              <Input
                id="endpoint"
                type="text"
                name="endpoint"
                defaultValue="/api/hello"
                placeholder="/api/hello"
                className="w-full"
              />
            </div>

            <Button type="submit" variant="secondary">
              Send
            </Button>
          </form>

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
