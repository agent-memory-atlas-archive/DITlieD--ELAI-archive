import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.jsx";
import "./app.css";

const rootEl = document.getElementById("root");
if (!rootEl) {
  throw new Error("mount target #root not found");
}
createRoot(rootEl).render(
  <StrictMode>
    <App />
  </StrictMode>
);
