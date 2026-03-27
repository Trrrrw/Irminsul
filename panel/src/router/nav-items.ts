import { LayoutDashboard, Settings, TestTubeDiagonal, type LucideIcon } from "lucide-react";

export type NavItem = {
  title: string;
  to: string;
  icon: LucideIcon;
};

export const navItems: NavItem[] = [
  {
    title: "概览",
    to: "/",
    icon: LayoutDashboard,
  },
  {
    title: "接口测试",
    to: "/api-tester",
    icon: TestTubeDiagonal,
  },
  {
    title: "系统设置",
    to: "/settings",
    icon: Settings,
  },
];
