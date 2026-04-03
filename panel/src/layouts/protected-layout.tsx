import { useEffect, useState, type FormEvent } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import { LoaderCircle, Save } from "lucide-react";

import {
  AdminSessionProvider,
  mapAdminProfileErrorMessage,
  useAdminSession,
  type AdminUser,
} from "@/components/app/admin-session-context";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Field, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

type AuthCheckState =
  | { status: "loading" }
  | { status: "authenticated"; user: AdminUser }
  | { status: "unauthenticated" }
  | { status: "error"; message: string };

type ProfileFormState = {
  username: string;
  email: string;
  currentPassword: string;
  newPassword: string;
};

function requiresProfileSetup(user: AdminUser) {
  return user.must_change_password || user.must_change_username || user.must_set_email;
}

function requiresEmail(user: AdminUser) {
  return user.must_set_email || !user.email;
}

export function ProtectedLayout() {
  const location = useLocation();
  const [authState, setAuthState] = useState<AuthCheckState>({ status: "loading" });

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

  return (
    <AdminSessionProvider
      user={authState.user}
      onUserChange={user => setAuthState({ status: "authenticated", user })}
      onUnauthenticated={() => setAuthState({ status: "unauthenticated" })}
    >
      <ProtectedLayoutContent mustRequireUsernameChange={mustRequireUsernameChange} />
    </AdminSessionProvider>
  );
}

type ProtectedLayoutContentProps = {
  mustRequireUsernameChange: boolean;
};

function ProtectedLayoutContent({ mustRequireUsernameChange }: ProtectedLayoutContentProps) {
  const { updateProfile, user } = useAdminSession();
  const [profileForm, setProfileForm] = useState<ProfileFormState>({
    username: user.username,
    email: user.email ?? "",
    currentPassword: "",
    newPassword: "",
  });
  const [profileError, setProfileError] = useState<string | null>(null);
  const [isSubmittingProfile, setIsSubmittingProfile] = useState(false);

  useEffect(() => {
    setProfileForm({
      username: user.username,
      email: user.email ?? "",
      currentPassword: "",
      newPassword: "",
    });
    setProfileError(null);
  }, [user.email, user.username]);

  async function handleProfileSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (isSubmittingProfile) {
      return;
    }

    const username = profileForm.username.trim();
    const email = profileForm.email.trim();
    const newPassword = profileForm.newPassword;
    const mustRequirePassword = user.must_change_password;
    const mustRequireEmail = requiresEmail(user);

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
      await updateProfile({
        username,
        email,
        ...(mustRequirePassword ? { newPassword } : {}),
      });
      setProfileForm(current => ({
        ...current,
        currentPassword: "",
        newPassword: "",
      }));
    } catch (error) {
      setProfileError(error instanceof Error ? error.message : mapAdminProfileErrorMessage());
    } finally {
      setIsSubmittingProfile(false);
    }
  }

  return (
    <>
      <Outlet />
      {requiresProfileSetup(user) ? (
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
                    (mustRequireUsernameChange && profileForm.username.trim() === user.username)
                  }
                >
                  <FieldLabel htmlFor="setup-username">用户名</FieldLabel>
                  <Input
                    id="setup-username"
                    type="text"
                    value={profileForm.username}
                    aria-invalid={
                      (Boolean(profileError) && !profileForm.username.trim()) ||
                      (mustRequireUsernameChange && profileForm.username.trim() === user.username)
                    }
                    onChange={event => setProfileForm(current => ({ ...current, username: event.target.value }))}
                  />
                </Field>

                <Field data-invalid={Boolean(profileError) && requiresEmail(user) && !profileForm.email.trim()}>
                  <FieldLabel htmlFor="setup-email">邮箱</FieldLabel>
                  <Input
                    id="setup-email"
                    type="email"
                    value={profileForm.email}
                    aria-invalid={Boolean(profileError) && requiresEmail(user) && !profileForm.email.trim()}
                    onChange={event => setProfileForm(current => ({ ...current, email: event.target.value }))}
                  />
                </Field>

                {user.must_change_password ? (
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
