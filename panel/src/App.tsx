import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { APITester } from "./APITester";
import { BrowserRouter, Routes, Link, Route } from "react-router-dom";
import "./index.css";

import logo from "./logo.svg";
import reactLogo from "./react.svg";
import { Button } from "./components/ui/button";

const Home = () => <div className="p-4"><h1>控制面板概览</h1></div>;
const Settings = () => <div className="p-4"><h1>系统设置</h1></div>;
const Layout = ({ children }: { children: React.ReactNode }) => (
  <div className="flex min-h-screen bg-background">
    {/* 侧边栏 */}
    <nav className="w-64 border-r p-4 flex flex-col gap-2">
      <div className="font-bold mb-4 text-xl px-4">Irminsul</div>
      <Button variant="ghost" asChild className="justify-start">
        <Link to="/">首页</Link>
      </Button>
      <Button variant="ghost" asChild className="justify-start">
        <Link to="/api-test">接口测试</Link>
      </Button>
      <Button variant="ghost" asChild className="justify-start">
        <Link to="/settings">设置</Link>
      </Button>
    </nav>

    {/* 主内容区 */}
    <main className="flex-1 overflow-auto">
      {children}
    </main>
  </div>
);

export function App() {
  return (
    <BrowserRouter basename="/admin">
      <Layout>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/api-test" element={<APITester />} />
          <Route path="/settings" element={<Settings />} />
          {/* 404 页面 */}
          <Route path="*" element={<div>404 - 页面不存在</div>} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}

export default App;
