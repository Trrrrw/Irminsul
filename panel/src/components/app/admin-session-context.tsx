import { createContext, useContext, useState, type PropsWithChildren } from "react";

const CSRF_STORAGE_KEY = "irminsul.admin.csrf-token";

export type AdminUser = {
  id: number;
  username: string;
  email: string | null;
  role: string;
  status: string;
  must_change_password: boolean;
  must_change_username: boolean;
  must_set_email: boolean;
};

type UpdateProfileInput = {
  username: string;
  email: string;
  currentPassword?: string;
  newPassword?: string;
};

type AdminSessionContextValue = {
  user: AdminUser;
  isLoggingOut: boolean;
  ensureCsrfToken: () => Promise<string>;
  updateProfile: (input: UpdateProfileInput) => Promise<AdminUser>;
  logout: () => Promise<void>;
};

type AdminSessionProviderProps = PropsWithChildren<{
  user: AdminUser;
  onUserChange: (user: AdminUser) => void;
  onUnauthenticated: () => void;
}>;

const AdminSessionContext = createContext<AdminSessionContextValue | null>(null);

type CsrfResponse = {
  csrf_token?: string;
  message?: string;
};

export function mapAdminProfileErrorMessage(message?: string) {
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
      return message ?? "操作失败，请稍后重试。";
  }
}

export function AdminSessionProvider({
  children,
  user,
  onUserChange,
  onUnauthenticated,
}: AdminSessionProviderProps) {
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  async function ensureCsrfToken() {
    const cachedToken = sessionStorage.getItem(CSRF_STORAGE_KEY);

    if (cachedToken) {
      return cachedToken;
    }

    const response = await fetch("/api/admin/auth/csrf", {
      method: "POST",
      credentials: "include",
    });

    const payload = (await response.json().catch(() => null)) as CsrfResponse | null;

    if (!response.ok || !payload?.csrf_token) {
      if (response.status === 401) {
        sessionStorage.removeItem(CSRF_STORAGE_KEY);
        onUnauthenticated();
      }

      throw new Error(mapAdminProfileErrorMessage(payload?.message));
    }

    sessionStorage.setItem(CSRF_STORAGE_KEY, payload.csrf_token);
    return payload.csrf_token;
  }

  async function updateProfile(input: UpdateProfileInput) {
    const csrfToken = await ensureCsrfToken();
    const response = await fetch("/api/admin/users/me", {
      method: "PATCH",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "X-CSRF-Token": csrfToken,
      },
      body: JSON.stringify({
        username: input.username,
        email: input.email,
        ...(input.currentPassword ? { current_password: input.currentPassword } : {}),
        ...(input.newPassword ? { new_password: input.newPassword } : {}),
      }),
    });

    const payload = (await response.json().catch(() => null)) as (AdminUser & { message?: string }) | null;

    if (!response.ok || !payload) {
      if (response.status === 401) {
        sessionStorage.removeItem(CSRF_STORAGE_KEY);
        onUnauthenticated();
      }

      throw new Error(mapAdminProfileErrorMessage(payload?.message));
    }

    onUserChange(payload);
    return payload;
  }

  async function logout() {
    setIsLoggingOut(true);

    try {
      const csrfToken = await ensureCsrfToken();
      const response = await fetch("/api/admin/auth/logout", {
        method: "POST",
        credentials: "include",
        headers: {
          "X-CSRF-Token": csrfToken,
        },
      });

      if (!response.ok && response.status !== 401) {
        const payload = (await response.json().catch(() => null)) as { message?: string } | null;
        throw new Error(mapAdminProfileErrorMessage(payload?.message));
      }

      sessionStorage.removeItem(CSRF_STORAGE_KEY);
      onUnauthenticated();
    } finally {
      setIsLoggingOut(false);
    }
  }

  return (
    <AdminSessionContext.Provider
      value={{
        user,
        isLoggingOut,
        ensureCsrfToken,
        updateProfile,
        logout,
      }}
    >
      {children}
    </AdminSessionContext.Provider>
  );
}

export function useAdminSession() {
  const context = useContext(AdminSessionContext);

  if (!context) {
    throw new Error("useAdminSession must be used within AdminSessionProvider");
  }

  return context;
}
