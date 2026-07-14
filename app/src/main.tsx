import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import "./wasm";
import { RouterProvider } from "react-router-dom";
import { router } from "./router.ts";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
