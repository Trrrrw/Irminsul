import { useRoutes, type RouteObject } from "react-router-dom";

import { AuthLayout } from "@/layouts/auth-layout";
import { ProtectedLayout } from "@/layouts/protected-layout";
import { LoginPage } from "@/pages/login-page";
import { RegisterPage } from "@/pages/register-page";

import { DashboardLayout } from "@/layouts/dashboard-layout";
import { ApiTesterPage } from "@/pages/api-tester-page";
import { NotFoundPage } from "@/pages/not-found-page";
import { OverviewPage } from "@/pages/overview-page";
import { SettingsPage } from "@/pages/settings-page";

export const appRoutes: RouteObject[] = [
  {
    path: "/login",
    element: <AuthLayout />,
    children: [{ index: true, element: <LoginPage /> }],
  },
  {
    path: "/register",
    element: <AuthLayout />,
    children: [{ index: true, element: <RegisterPage /> }],
  },
  {
    path: "/",
    element: <ProtectedLayout />,
    children: [
      {
        element: <DashboardLayout />,
        children: [
          {
            index: true,
            element: <OverviewPage />,
          },
          {
            path: "api-tester",
            element: <ApiTesterPage />,
          },
          {
            path: "settings",
            element: <SettingsPage />,
          },
          {
            path: "*",
            element: <NotFoundPage />,
          },
        ],
      },
    ],
  },
];

export function AppRoutes() {
  return useRoutes(appRoutes);
}
