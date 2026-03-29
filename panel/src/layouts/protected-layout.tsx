import { useEffect, useState, type FormEvent } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import { LoaderCircle, Save } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Field, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

const CSRF_STORAGE_KEY = "irminsul.admin.csrf-token";

type AdminUser = {
  id: number;
  username: string;
  email: string | null;
  must_change_password: boolean;
  must_change_username: boolean;
  must_set_email: boolean;
};

type AuthCheckState =
  | { status: "loading" }
  | { status: "authenticated"; user: AdminUser }
  | { status: "unauthenticated" }
  | { status: "error"; message: string };

type ProfileFormState = {
  username: string;
  email: string;
  newPassword: string;
};

type CsrfResponse = {
  csrf_token?: string;
};

function requiresProfileSetup(user: AdminUser) {
  return user.must_change_password || user.must_change_username || user.must_set_email;
}

function requiresEmail(user: AdminUser) {
  return user.must_set_email || !user.email;
}

function mapProfileSetupErrorMessage(message?: string) {
  switch (message) {
    case "username change is required":
      return "请修改用户名后再继续。";
    case "username is required":
      return "请输入用户名。";
    case "password change is required":
      return "请设置新密码后再继续。";
    case "email is required":
      return "请输入邮箱。";
    case "current password is required":
      return "当前密码不能为空。";
    case "current password is invalid":
      return "当前密码不正确。";
    case "password does not satisfy the policy":
      return "密码不符合后端校验规则。";
    case "failed to update profile":
      return "资料更新失败，请检查是否与现有账户冲突。";
    case "authentication required":
      return "登录已失效，请重新登录。";
    case "invalid JSON payload":
      return "请求格式不正确。";
    default:
      return message ?? "资料更新失败，请稍后重试。";
  }
}

export function ProtectedLayout() {
  const location = useLocation();
  const [authState, setAuthState] = useState<AuthCheckState>({ status: "loading" });
  const [profileForm, setProfileForm] = useState<ProfileFormState>({
    username: "",
    email: "",
    newPassword: "",
  });
  const [profileError, setProfileError] = useState<string | null>(null);
  const [isSubmittingProfile, setIsSubmittingProfile] = useState(false);

  useEffect(() => {
    const controller = new AbortController();

    async function checkSession() {
      try {
        const response = await fetch("/api/admin/auth/me", {
          credentials: "include",
          signal: controller.signal,
        });

        if (response.ok) {
          const user = (await response.json()) as AdminUser;
          setAuthState({ status: "authenticated", user });
          setProfileForm({
            username: user.username,
            email: user.email ?? "",
            newPassword: "",
          });
          setProfileError(null);
          return;
        }

        if (response.status === 401) {
          setAuthState({ status: "unauthenticated" });
          return;
        }

        setAuthState({ status: "error", message: "登录状态检查失败，请稍后重试。" });
      } catch (error) {
        if (controller.signal.aborted) {
          return;
        }

        setAuthState({
          status: "error",
          message: error instanceof Error ? error.message : "登录状态检查失败，请稍后重试。",
        });
      }
    }

    void checkSession();

    return () => {
      controller.abort();
    };
  }, []);

  async function ensureCsrfToken() {
    const cachedToken = sessionStorage.getItem(CSRF_STORAGE_KEY);

    if (cachedToken) {
      return cachedToken;
    }

    const response = await fetch("/api/admin/auth/csrf", {
      method: "POST",
      credentials: "include",
    });

    const payload = (await response.json().catch(() => null)) as CsrfResponse & { message?: string } | null;

    if (!response.ok || !payload?.csrf_token) {
      if (response.status === 401) {
        setAuthState({ status: "unauthenticated" });
      }
      throw new Error(mapProfileSetupErrorMessage(payload?.message));
    }

    sessionStorage.setItem(CSRF_STORAGE_KEY, payload.csrf_token);
    return payload.csrf_token;
  }

  async function handleProfileSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (authState.status !== "authenticated" || isSubmittingProfile) {
      return;
    }

    const username = profileForm.username.trim();
    const email = profileForm.email.trim();
    const newPassword = profileForm.newPassword;
    const mustRequirePassword = authState.user.must_change_password;
    const mustRequireUsernameChange = authState.user.must_change_username;
    const mustRequireEmail = requiresEmail(authState.user);

    if (!username) {
      setProfileError("请输入用户名。");
      return;
    }

    if (mustRequireEmail && !email) {
      setProfileError("请输入邮箱。");
      return;
    }

    if (mustRequirePassword && !newPassword) {
      setProfileError("请设置新密码。");
      return;
    }

    setIsSubmittingProfile(true);
    setProfileError(null);

    try {
      const csrfToken = await ensureCsrfToken();
      const response = await fetch("/api/admin/users/me", {
        method: "PATCH",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
          "X-CSRF-Token": csrfToken,
        },
        body: JSON.stringify({
          username,
          email,
          ...(mustRequirePassword ? { new_password: newPassword } : {}),
        }),
      });

      const payload = (await response.json().catch(() => null)) as AdminUser & { message?: string } | null;

      if (!response.ok || !payload) {
        if (response.status === 401) {
          setAuthState({ status: "unauthenticated" });
          return;
        }

        throw new Error(mapProfileSetupErrorMessage(payload?.message));
      }

      setAuthState({ status: "authenticated", user: payload });
      setProfileForm({
        username: payload.username,
        email: payload.email ?? "",
        newPassword: "",
      });
      setProfileError(null);
    } catch (error) {
      setProfileError(error instanceof Error ? error.message : "资料更新失败，请稍后重试。");
    } finally {
      setIsSubmittingProfile(false);
    }
  }

  if (authState.status === "loading") {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background px-6 py-10">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-xl">
              <LoaderCircle className="animate-spin" />
              正在验证登录状态
            </CardTitle>
            <CardDescription>请稍候，系统正在确认当前后台会话。</CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  if (authState.status === "unauthenticated") {
    return (
      <Navigate
        to="/login"
        replace
        state={{
          from: `${location.pathname}${location.search}${location.hash}`,
        }}
      />
    );
  }

  if (authState.status === "error") {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background px-6 py-10">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="text-xl">无法进入后台</CardTitle>
            <CardDescription>{authState.message}</CardDescription>
          </CardHeader>
          <CardContent className="text-sm leading-6 text-muted-foreground">
            如果你刚刚登录过，请刷新页面重试；如果 session 已失效，系统会在下次检查时跳回登录页。
          </CardContent>
        </Card>
      </div>
    );
  }

  const mustRequireUsernameChange = authState.user.must_change_username;
  const mustRequireEmail = requiresEmail(authState.user);
  const mustRequirePassword = authState.user.must_change_password;

  return (
    <>
      <Outlet />
      {requiresProfileSetup(authState.user) ? (
        <Dialog open onOpenChange={() => undefined}>
          <DialogContent
            showCloseButton={false}
            className="sm:max-w-xl"
            onEscapeKeyDown={event => event.preventDefault()}
            onInteractOutside={event => event.preventDefault()}
            onPointerDownOutside={event => event.preventDefault()}
          >
            <DialogHeader>
              <DialogTitle>完善账户信息</DialogTitle>
              <DialogDescription>请先完成当前账户所需的资料补全，完成后才能继续使用后台。</DialogDescription>
            </DialogHeader>

            <form onSubmit={handleProfileSubmit} className="flex flex-col gap-6">
              <FieldGroup>
                <Field
                  data-invalid={
                    (Boolean(profileError) && !profileForm.username.trim()) ||
                    (mustRequireUsernameChange && profileForm.username.trim() === authState.user.username)
                  }
                >
                  <FieldLabel htmlFor="setup-username">用户名</FieldLabel>
                  <Input
                    id="setup-username"
                    type="text"
                    value={profileForm.username}
                    aria-invalid={
                      (Boolean(profileError) && !profileForm.username.trim()) ||
                      (mustRequireUsernameChange && profileForm.username.trim() === authState.user.username)
                    }
                    onChange={event => setProfileForm(current => ({ ...current, username: event.target.value }))}
                  />
                </Field>

                <Field data-invalid={Boolean(profileError) && mustRequireEmail && !profileForm.email.trim()}>
                  <FieldLabel htmlFor="setup-email">邮箱</FieldLabel>
                  <Input
                    id="setup-email"
                    type="email"
                    value={profileForm.email}
                    aria-invalid={Boolean(profileError) && mustRequireEmail && !profileForm.email.trim()}
                    onChange={event => setProfileForm(current => ({ ...current, email: event.target.value }))}
                  />
                </Field>

                {mustRequirePassword ? (
                  <Field data-invalid={Boolean(profileError) && !profileForm.newPassword}>
                    <FieldLabel htmlFor="setup-password">新密码</FieldLabel>
                    <Input
                      id="setup-password"
                      type="password"
                      value={profileForm.newPassword}
                      aria-invalid={Boolean(profileError) && !profileForm.newPassword}
                      onChange={event => setProfileForm(current => ({ ...current, newPassword: event.target.value }))}
                    />
                  </Field>
                ) : null}
              </FieldGroup>

              {profileError ? <FieldError>{profileError}</FieldError> : null}

              <Button type="submit" disabled={isSubmittingProfile}>
                {isSubmittingProfile ? (
                  <LoaderCircle data-icon="inline-start" className="animate-spin" />
                ) : (
                  <Save data-icon="inline-start" />
                )}
                {isSubmittingProfile ? "正在保存" : "保存并继续"}
              </Button>
            </form>
          </DialogContent>
        </Dialog>
      ) : null}
    </>
  );
}
