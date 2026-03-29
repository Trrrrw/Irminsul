import { startTransition, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { LoaderCircle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

const CSRF_STORAGE_KEY = "irminsul.admin.csrf-token";

type RegisterFormState = {
  invitationToken: string;
  username: string;
  email: string;
  password: string;
  confirmPassword: string;
};

type RegisterResponse = {
  csrf_token?: string;
};

function mapRegisterErrorMessage(message?: string) {
  switch (message) {
    case "registration request is invalid":
      return "注册信息无效，请检查邀请码、用户名、邮箱和密码。";
    case "invitation does not exist":
      return "邀请码不存在。";
    case "invitation has expired":
      return "邀请码已过期。";
    case "invitation is no longer valid":
      return "邀请码已失效。";
    case "invalid JSON payload":
      return "请求格式不正确。";
    default:
      return message ?? "注册失败，请稍后重试。";
  }
}

export function RegisterPage() {
  const navigate = useNavigate();
  const [formState, setFormState] = useState<RegisterFormState>({
    invitationToken: "",
    username: "",
    email: "",
    password: "",
    confirmPassword: "",
  });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (isSubmitting) {
      return;
    }

    const invitationToken = formState.invitationToken.trim();
    const username = formState.username.trim();
    const email = formState.email.trim();
    const password = formState.password;
    const confirmPassword = formState.confirmPassword;

    if (!invitationToken || !username || !email || !password || !confirmPassword) {
      setErrorMessage("请填写完整的注册信息。");
      return;
    }

    if (password !== confirmPassword) {
      setErrorMessage("两次输入的密码不一致。");
      return;
    }

    setIsSubmitting(true);
    setErrorMessage(null);

    try {
      const response = await fetch("/api/admin/auth/register", {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          invitation_token: invitationToken,
          username,
          email,
          password,
        }),
      });

      const payload = (await response.json().catch(() => null)) as RegisterResponse & { message?: string } | null;

      if (!response.ok) {
        throw new Error(mapRegisterErrorMessage(payload?.message));
      }

      if (payload?.csrf_token) {
        sessionStorage.setItem(CSRF_STORAGE_KEY, payload.csrf_token);
      }

      startTransition(() => {
        navigate("/", { replace: true });
      });
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : "注册失败，请稍后重试。");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Card className="mx-auto w-full max-w-xl border-border/70 bg-card/90 shadow-xl backdrop-blur">
      <CardHeader className="gap-3">
        <CardTitle className="text-2xl">邀请码注册</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="flex flex-col gap-6">
          <FieldGroup>
            <Field data-invalid={Boolean(errorMessage) && !formState.invitationToken.trim()}>
              <FieldLabel htmlFor="invitation-token">邀请码</FieldLabel>
              <Input
                id="invitation-token"
                type="text"
                value={formState.invitationToken}
                aria-invalid={Boolean(errorMessage) && !formState.invitationToken.trim()}
                onChange={event => setFormState(current => ({ ...current, invitationToken: event.target.value }))}
              />
            </Field>

            <Field data-invalid={Boolean(errorMessage) && !formState.username.trim()}>
              <FieldLabel htmlFor="register-username">用户名</FieldLabel>
              <Input
                id="register-username"
                type="text"
                value={formState.username}
                aria-invalid={Boolean(errorMessage) && !formState.username.trim()}
                onChange={event => setFormState(current => ({ ...current, username: event.target.value }))}
              />
            </Field>

            <Field data-invalid={Boolean(errorMessage) && !formState.email.trim()}>
              <FieldLabel htmlFor="register-email">邮箱</FieldLabel>
              <Input
                id="register-email"
                type="email"
                value={formState.email}
                aria-invalid={Boolean(errorMessage) && !formState.email.trim()}
                onChange={event => setFormState(current => ({ ...current, email: event.target.value }))}
              />
            </Field>

            <Field data-invalid={Boolean(errorMessage) && !formState.password}>
              <FieldLabel htmlFor="register-password">密码</FieldLabel>
              <Input
                id="register-password"
                type="password"
                value={formState.password}
                aria-invalid={Boolean(errorMessage) && !formState.password}
                onChange={event => setFormState(current => ({ ...current, password: event.target.value }))}
              />
            </Field>

            <Field data-invalid={Boolean(errorMessage) && (!formState.confirmPassword || formState.password !== formState.confirmPassword)}>
              <FieldLabel htmlFor="register-confirm-password">确认密码</FieldLabel>
              <Input
                id="register-confirm-password"
                type="password"
                value={formState.confirmPassword}
                aria-invalid={
                  Boolean(errorMessage) && (!formState.confirmPassword || formState.password !== formState.confirmPassword)
                }
                onChange={event => setFormState(current => ({ ...current, confirmPassword: event.target.value }))}
              />
            </Field>
          </FieldGroup>

          {errorMessage ? <FieldError>{errorMessage}</FieldError> : null}

          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? <LoaderCircle data-icon="inline-start" className="animate-spin" /> : null}
            {isSubmitting ? "正在注册" : "注册并进入后台"}
          </Button>
        </form>
      </CardContent>
      <CardFooter className="border-t border-border/70 pt-6">
        <Button asChild variant="link" className="px-0">
          <Link to="/login">返回登录</Link>
        </Button>
      </CardFooter>
    </Card>
  );
}
