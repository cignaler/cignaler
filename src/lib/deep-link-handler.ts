/**
 * Deep link handler for cignaler:// URIs.
 *
 * Handles deep links on both platforms:
 * - macOS: onOpenUrl from @tauri-apps/plugin-deep-link
 * - Windows/Linux: listen for 'deep-link-received' Tauri event (from single-instance plugin)
 *
 * URI format: cignaler://add-watcher?name=<name>&project=<project>&ref=<ref>&ci_server=<ci_server>
 */

import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { listen } from "@tauri-apps/api/event";
import { addWatcher } from "./stores/watchers.svelte";
import { toast } from "./stores/toast.svelte";

interface DeepLinkParams {
  name: string;
  project: string;
  ref: string;
  ci_server: string;
}

function parseDeepLink(urlString: string): DeepLinkParams | null {
  try {
    // cignaler://add-watcher?name=...&project=...&ref=...&ci_server=...
    // URL constructor needs a valid scheme — cignaler:// works
    const url = new URL(urlString);

    // The "host" for cignaler://add-watcher is "add-watcher"
    if (url.hostname !== "add-watcher") {
      console.warn("Unknown deep link action:", url.hostname);
      return null;
    }

    const name = url.searchParams.get("name");
    const project = url.searchParams.get("project");
    const ref = url.searchParams.get("ref");
    const ci_server = url.searchParams.get("ci_server");

    if (!name || !project || !ref || !ci_server) {
      console.warn("Missing deep link params:", { name, project, ref, ci_server });
      return null;
    }

    return { name, project, ref, ci_server };
  } catch (err) {
    console.error("Failed to parse deep link URL:", err);
    return null;
  }
}

async function handleDeepLink(urlString: string) {
  console.log("Handling deep link:", urlString);

  const params = parseDeepLink(urlString);
  if (!params) {
    toast.error("Invalid deep link received.");
    return;
  }

  try {
    await addWatcher(params.name, params.ci_server, params.project, params.ref);
    toast.success(`Watcher "${params.name}" added from deep link.`);
  } catch (err) {
    console.error("Failed to add watcher from deep link:", err);
    toast.error("Failed to add watcher: " + String(err));
  }
}

/**
 * Initialize deep link listeners. Call once on app startup.
 */
export function initDeepLinkHandler() {
  // macOS: onOpenUrl fires when the OS opens a cignaler:// link
  onOpenUrl((urls: string[]) => {
    for (const url of urls) {
      handleDeepLink(url);
    }
  }).catch((err) => {
    console.error("Failed to register deep link handler:", err);
  });

  // Windows/Linux: single-instance plugin emits this event with the URL
  listen<string>("deep-link-received", (event) => {
    handleDeepLink(event.payload);
  }).catch((err) => {
    console.error("Failed to listen for deep-link-received event:", err);
  });
}
