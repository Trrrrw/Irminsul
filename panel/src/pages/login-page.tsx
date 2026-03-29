import { startTransition, useState, type FormEvent } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { LoaderCircle, LogIn } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

const CSRF_STORAGE_KEY = "irminsul.admin.csrf-token";

type LoginFormState = {
  identifier: string;
  password: string;
};

type LoginResponse = {
  csrf_token?: string;
};

function mapLoginErrorMessage(message?: string) {
  switch (message) {
    case "invalid credentials":
      return "用户名、邮箱或密码错误。";
    case "account is disabled":
      return "账号已被禁用。";
    case "too many failed login attempts":
      return "登录失败次数过多，请稍后再试。";
    case "invalid JSON payload":
      return "请求格式不正确。";
    default:
      return message ?? "登录失败，请检查账户信息后重试。";
  }
}

export function LoginPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const [formState, setFormState] = useState<LoginFormState>({ identifier: "", password: "" });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const redirectTo =
    typeof location.state?.from === "string" && location.state.from.startsWith("/") ? location.state.from : "/";

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (isSubmitting) {
      return;
    }

    const trimmedIdentifier = formState.identifier.trim();
    const password = formState.password;

    if (!trimmedIdentifier || !password) {
      setErrorMessage("请输入用户名或邮箱，以及密码。");
      return;
    }

    setIsSubmitting(true);
    setErrorMessage(null);

    try {
      const response = await fetch("/api/admin/auth/login", {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          identifier: trimmedIdentifier,
          password,
        }),
      });

      const payload = (await response.json().catch(() => null)) as LoginResponse & { message?: string } | null;

      if (!response.ok) {
        throw new Error(mapLoginErrorMessage(payload?.message));
      }

      if (payload?.csrf_token) {
        sessionStorage.setItem(CSRF_STORAGE_KEY, payload.csrf_token);
      }

      startTransition(() => {
        navigate(redirectTo, { replace: true });
      });
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : "登录失败，请稍后重试。");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card className="mx-auto w-full max-w-xl border-border/70 bg-card/90 shadow-xl backdrop-blur">
      <CardHeader className="gap-3">
        <CardTitle className="text-2xl">登录后台</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="flex flex-col gap-6">
          <FieldGroup>
            <Field data-invalid={Boolean(errorMessage) && !formState.identifier.trim()}>
              <FieldLabel htmlFor="identifier">用户名或邮箱</FieldLabel>
              <Input
                id="identifier"
                name="identifier"
                type="text"
                autoComplete="username"
                value={formState.identifier}
                aria-invalid={Boolean(errorMessage) && !formState.identifier.trim()}
                placeholder="admin 或 admin@example.com"
                onChange={event => setFormState(current => ({ ...current, identifier: event.target.value }))}
              />
            </Field>

            <Field data-invalid={Boolean(errorMessage) && !formState.password}>
              <FieldLabel htmlFor="password">密码</FieldLabel>
              <Input
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                value={formState.password}
                aria-invalid={Boolean(errorMessage) && !formState.password}
                placeholder="请输入密码"
                onChange={event => setFormState(current => ({ ...current, password: event.target.value }))}
              />
            </Field>
          </FieldGroup>

          {errorMessage ? <FieldError>{errorMessage}</FieldError> : null}

          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? <LoaderCircle data-icon="inline-start" className="animate-spin" /> : <LogIn data-icon="inline-start" />}
            {isSubmitting ? "正在登录" : "登录"}
          </Button>
        </form>
      </CardContent>
      <CardFooter className="justify-between gap-3 border-t border-border/70 pt-6 text-sm text-muted-foreground">
        <Button asChild variant="link" className="px-0">
          <Link to="/register">邀请码注册</Link>
        </Button>
      </CardFooter>
    </Card>
  );
}
