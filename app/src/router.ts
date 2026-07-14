import { createBrowserRouter } from "react-router-dom";
import RootPage from "./app/Root.page";

export const router = createBrowserRouter([
  {
    path: "/",
    Component: RootPage,
  },
]);
