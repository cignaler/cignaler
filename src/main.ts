import "@fontsource-variable/outfit";
import "@fontsource-variable/jetbrains-mono";
import "./app.css";
import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import App from "./App.svelte";

const appEl = document.getElementById("app")!;
appEl.innerHTML = "";
const app = mount(App, { target: appEl });

// Show window now that Svelte has mounted
invoke("show_main_window").catch((e) =>
  console.error("Failed to show window:", e),
);

export default app;
