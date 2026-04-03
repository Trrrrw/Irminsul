export const accountMenuItems = [
  { id: "settings", label: "系统设置" },
  { id: "logout", label: "退出登录", tone: "danger" as const },
] as const;

export const settingsSections = [
  { id: "general", label: "通用设置" },
  { id: "account", label: "账号管理" },
] as const;

export const generalSettingsItems = [
  { id: "language", label: "语言" },
  { id: "theme", label: "主题" },
] as const;

export const themeOptions = [
  { id: "light", label: "浅色" },
  { id: "dark", label: "深色" },
  { id: "system", label: "跟随系统" },
] as const;

export const languageOptions = [
  { id: "zh-CN", label: "简体中文" },
  { id: "en-US", label: "English" },
  { id: "ja-JP", label: "日本語" },
] as const;

export type AccountMenuItemId = typeof accountMenuItems[number]["id"];
export type SettingsSectionId = typeof settingsSections[number]["id"];
export type GeneralSettingsItemId = typeof generalSettingsItems[number]["id"];
export type ThemeMode = typeof themeOptions[number]["id"];
export type LanguageOptionId = typeof languageOptions[number]["id"];

export function getAccountBadgeLabel(role: string) {
  switch (role) {
    case "owner":
      return "所有者";
    case "admin":
      return "管理员";
    default:
      return "管理员";
  }
}

export function getAccountInitials(username: string) {
  const asciiCharacters = username.replace(/[^A-Za-z0-9]+/g, "").slice(0, 2);

  if (asciiCharacters) {
    return asciiCharacters.toUpperCase();
  }

  return username.trim().slice(0, 1).toUpperCase();
}
