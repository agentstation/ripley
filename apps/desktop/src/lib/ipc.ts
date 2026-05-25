import { listen, type UnlistenFn, type EventCallback } from "@tauri-apps/api/event";

export function subscribe<T>(channel: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  return listen<T>(channel, handler);
}
