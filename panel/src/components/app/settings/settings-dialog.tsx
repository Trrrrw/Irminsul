import { useEffect, useState, type FormEvent } from "react";
import {
  Languages,
  LoaderCircle,
  Mail,
  Monitor,
  MoonStar,
  Save,
  Settings2,
  ShieldCheck,
  Sun,
  UserRound,
} from "lucide-react";

import {
  generalSettingsItems,
  getAccountBadgeLabel,
  languageOptions,
  settingsSections,
  themeOptions,
  type LanguageOptionId,
  type SettingsSectionId,
  type ThemeMode,
} from "@/components/app/account-shell";
import { useAdminSession, type AdminUser } from "@/components/app/admin-session-context";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { FieldError } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

const LANGUAGE_STORAGE_KEY = "irminsul.admin.language";
const THEME_STORAGE_KEY = "irminsul.admin.theme";

type AccountFormState = {
  username: string;
  email: string;
  currentPassword: string;
  newPassword: string;
};

type SettingsDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialSection?: SettingsSectionId;
  initialAccountError?: string | null;
};

export const SETTINGS_DIALOG_CLASS_NAMES = {
  content:
    "max-h-[calc(100dvh-1rem)] overflow-y-auto border-border/70 bg-card/95 p-0 shadow-2xl sm:max-w-4xl",
  layout: "grid min-h-0 md:min-h-[36rem] md:grid-cols-[15rem_minmax(0,1fr)]",
  generalSettingRow: "flex flex-col items-start gap-3 px-5 py-3.5 md:flex-row md:items-center md:justify-between md:gap-4",
  generalSettingTitle: "flex items-center gap-2 whitespace-nowrap text-sm font-medium",
  generalSettingControl: "w-full md:w-40",
} as const;

function readStoredValue<T extends string>(key: string, fallback: T) {
  if (typeof window === "undefined") {
    return fallback;
  }

  const storedValue = window.localStorage.getItem(key) as T | null;
  return storedValue ?? fallback;
}

export function buildAccountFormState(user: AdminUser): AccountFormState {
  return {
    username: user.username,
    email: user.email ?? "",
    currentPassword: "",
    newPassword: "",
  };
}

export function resolveThemeAppearance(mode: ThemeMode, prefersDark: boolean) {
  if (mode === "light") {
    return "light";
  }

  if (mode === "dark") {
    return "dark";
  }

  return prefersDark ? "dark" : "light";
}

function resolveTheme(mode: ThemeMode) {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return resolveThemeAppearance(mode, false);
  }

  return resolveThemeAppearance(mode, window.matchMedia("(prefers-color-scheme: dark)").matches);
}

export function SettingsDialog({
  open,
  onOpenChange,
  initialSection = "general",
  initialAccountError = null,
}: SettingsDialogProps) {
  const { isLoggingOut, updateProfile, user } = useAdminSession();
  const [activeSettingsSection, setActiveSettingsSection] = useState<SettingsSectionId>(initialSection);
  const [language, setLanguage] = useState<LanguageOptionId>(() => readStoredValue(LANGUAGE_STORAGE_KEY, "zh-CN"));
  const [themeMode, setThemeMode] = useState<ThemeMode>(() => readStoredValue(THEME_STORAGE_KEY, "system"));
  const [accountError, setAccountError] = useState<string | null>(initialAccountError);
  const [isSavingAccount, setIsSavingAccount] = useState(false);
  const [accountForm, setAccountForm] = useState<AccountFormState>(() => buildAccountFormState(user));

  useEffect(() => {
    setAccountForm(buildAccountFormState(user));
    setAccountError(initialAccountError);
  }, [initialAccountError, user]);

  useEffect(() => {
    if (!open) {
      return;
    }

    setActiveSettingsSection(initialSection);
    setAccountError(initialAccountError);
  }, [initialAccountError, initialSection, open]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
  }, [language]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem(THEME_STORAGE_KEY, themeMode);

    const mediaQuery = typeof window.matchMedia === "function"
      ? window.matchMedia("(prefers-color-scheme: dark)")
      : null;

    const syncTheme = () => {
      document.documentElement.classList.toggle("dark", resolveTheme(themeMode) === "dark");
    };

    syncTheme();

    if (!mediaQuery) {
      return;
    }

    if (typeof mediaQuery.addEventListener === "function") {
      mediaQuery.addEventListener("change", syncTheme);
      return () => mediaQuery.removeEventListener("change", syncTheme);
    }

    mediaQuery.addListener(syncTheme);
    return () => mediaQuery.removeListener(syncTheme);
  }, [themeMode]);

  const currentLanguage = languageOptions.find(option => option.id === language) ?? languageOptions[0];
  const currentTheme = themeOptions.find(option => option.id === themeMode) ?? themeOptions[2];
  const resolvedTheme = resolveTheme(themeMode);

  async function handleAccountSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (isSavingAccount) {
      return;
    }

    const username = accountForm.username.trim();
    const email = accountForm.email.trim();

    if (!username) {
      setAccountError("请输入用户名。");
      return;
    }

    if (!email) {
      setAccountError("请输入邮箱。");
      return;
    }

    if (accountForm.newPassword && !accountForm.currentPassword) {
      setAccountError("如需修改密码，请先输入当前密码。");
      return;
    }

    setIsSavingAccount(true);
    setAccountError(null);

    try {
      await updateProfile({
        username,
        email,
        ...(accountForm.currentPassword ? { currentPassword: accountForm.currentPassword } : {}),
        ...(accountForm.newPassword ? { newPassword: accountForm.newPassword } : {}),
      });
      setAccountForm(current => ({
        ...current,
        currentPassword: "",
        newPassword: "",
      }));
    } catch (error) {
      setAccountError(error instanceof Error ? error.message : "账号资料更新失败，请稍后重试。");
    } finally {
      setIsSavingAccount(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={SETTINGS_DIALOG_CLASS_NAMES.content}>
        <div className={SETTINGS_DIALOG_CLASS_NAMES.layout}>
          <section className="border-b border-border/70 bg-muted/30 p-4 md:border-r md:border-b-0">
            <DialogHeader className="mb-5 px-0">
              <DialogTitle>系统设置</DialogTitle>
            </DialogHeader>

            <nav className="flex flex-col gap-1.5">
              {settingsSections.map(section => (
                <button
                  key={section.id}
                  type="button"
                  className={
                    activeSettingsSection === section.id
                      ? "flex items-center gap-2 rounded-xl bg-primary/10 px-3 py-2.5 text-left text-sm font-medium text-primary transition-colors"
                      : "flex items-center gap-2 rounded-xl px-3 py-2.5 text-left text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                  }
                  onClick={() => setActiveSettingsSection(section.id)}
                >
                  {section.id === "general" ? <Settings2 className="size-4" /> : <ShieldCheck className="size-4" />}
                  <span>{section.label}</span>
                </button>
              ))}
            </nav>
          </section>

          <section className="flex min-h-0 flex-col p-6">
            {activeSettingsSection === "general" ? (
              <div className="flex flex-col gap-5">
                <header>
                  <h3 className="text-xl font-semibold tracking-tight text-foreground">通用设置</h3>
                </header>

                <div className="grid gap-2.5">
                  {generalSettingsItems.map(item => (
                    <Card key={item.id} className="gap-0 border-border/70 py-0 shadow-none">
                      <CardContent className={SETTINGS_DIALOG_CLASS_NAMES.generalSettingRow}>
                        <CardTitle className={SETTINGS_DIALOG_CLASS_NAMES.generalSettingTitle}>
                          {item.id === "language" ? <Languages className="size-4" /> : <Monitor className="size-4" />}
                          {item.label}
                        </CardTitle>

                        {item.id === "language" ? (
                          <Select value={language} onValueChange={value => setLanguage(value as LanguageOptionId)}>
                            <SelectTrigger size="sm" className={SETTINGS_DIALOG_CLASS_NAMES.generalSettingControl}>
                              <SelectValue placeholder="选择语言" />
                            </SelectTrigger>
                            <SelectContent align="end">
                              {languageOptions.map(option => (
                                <SelectItem key={option.id} value={option.id}>
                                  {option.label}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : (
                          <Select value={themeMode} onValueChange={value => setThemeMode(value as ThemeMode)}>
                            <SelectTrigger size="sm" className={SETTINGS_DIALOG_CLASS_NAMES.generalSettingControl}>
                              <SelectValue placeholder="选择主题" />
                            </SelectTrigger>
                            <SelectContent align="end">
                              {themeOptions.map(option => (
                                <SelectItem key={option.id} value={option.id}>
                                  {option.id === "light" ? (
                                    <span className="flex items-center gap-2">
                                      <Sun className="size-4" />
                                      {option.label}
                                    </span>
                                  ) : option.id === "dark" ? (
                                    <span className="flex items-center gap-2">
                                      <MoonStar className="size-4" />
                                      {option.label}
                                    </span>
                                  ) : (
                                    <span className="flex items-center gap-2">
                                      <Monitor className="size-4" />
                                      {option.label}
                                    </span>
                                  )}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        )}
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </div>
            ) : (
              <div className="flex flex-col gap-6">
                <header>
                  <h3 className="text-xl font-semibold tracking-tight text-foreground">账号管理</h3>
                </header>

                <div className="grid gap-3 md:grid-cols-3">
                  <div className="rounded-2xl border border-border/70 bg-muted/25 p-4">
                    <div className="mb-3 inline-flex rounded-xl bg-primary/10 p-2 text-primary">
                      <UserRound className="size-4" />
                    </div>
                    <div className="text-sm font-medium text-foreground">用户名</div>
                    <div className="mt-1 text-sm text-muted-foreground">{user.username}</div>
                  </div>
                  <div className="rounded-2xl border border-border/70 bg-muted/25 p-4">
                    <div className="mb-3 inline-flex rounded-xl bg-primary/10 p-2 text-primary">
                      <Mail className="size-4" />
                    </div>
                    <div className="text-sm font-medium text-foreground">邮箱</div>
                    <div className="mt-1 text-sm text-muted-foreground">{user.email ?? "未设置"}</div>
                  </div>
                  <div className="rounded-2xl border border-border/70 bg-muted/25 p-4">
                    <div className="mb-3 inline-flex rounded-xl bg-primary/10 p-2 text-primary">
                      <ShieldCheck className="size-4" />
                    </div>
                    <div className="text-sm font-medium text-foreground">角色</div>
                    <div className="mt-1 text-sm text-muted-foreground">{getAccountBadgeLabel(user.role)}</div>
                  </div>
                </div>

                <form onSubmit={handleAccountSubmit} className="grid gap-4 rounded-3xl border border-border/70 bg-card p-5 shadow-none">
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label htmlFor="account-username">用户名</Label>
                      <Input
                        id="account-username"
                        value={accountForm.username}
                        onChange={event => setAccountForm(current => ({ ...current, username: event.target.value }))}
                      />
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="account-email">邮箱</Label>
                      <Input
                        id="account-email"
                        type="email"
                        value={accountForm.email}
                        onChange={event => setAccountForm(current => ({ ...current, email: event.target.value }))}
                      />
                    </div>
                  </div>

                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label htmlFor="account-current-password">当前密码</Label>
                      <Input
                        id="account-current-password"
                        type="password"
                        value={accountForm.currentPassword}
                        onChange={event =>
                          setAccountForm(current => ({ ...current, currentPassword: event.target.value }))
                        }
                      />
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="account-new-password">新密码</Label>
                      <Input
                        id="account-new-password"
                        type="password"
                        value={accountForm.newPassword}
                        onChange={event => setAccountForm(current => ({ ...current, newPassword: event.target.value }))}
                      />
                    </div>
                  </div>

                  {accountError ? <FieldError>{accountError}</FieldError> : null}

                  <div className="flex justify-end">
                    <Button type="submit" disabled={isSavingAccount || isLoggingOut}>
                      {isSavingAccount ? (
                        <LoaderCircle data-icon="inline-start" className="animate-spin" />
                      ) : (
                        <Save data-icon="inline-start" />
                      )}
                      {isSavingAccount ? "正在保存" : "保存更改"}
                    </Button>
                  </div>
                </form>
              </div>
            )}
          </section>
        </div>
      </DialogContent>
    </Dialog>
  );
}
