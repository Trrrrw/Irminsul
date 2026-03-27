import { BrowserRouter } from "react-router-dom";

import { AppRoutes } from "@/router/routes";

import "./index.css";

export function App() {
  return (
    <BrowserRouter basename="/admin">
      <AppRoutes />
    </BrowserRouter>
  );
}

export default App;
