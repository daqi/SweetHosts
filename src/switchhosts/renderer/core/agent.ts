/**
 * agent
 * @author: oldj
 * @homepage: https://oldj.net
 */

import { Actions } from "@common/types";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import events from '@common/events';


function toSnake(s: string) {
  return s.replace(/([A-Z])/g, "_$1").toLowerCase();
}

function tryParseJSON(v: any) {
  if (typeof v !== "string") return v;
  try {
    return JSON.parse(v);
  } catch (e) {
    return v;
  }
}

class TauriAgent {
  async call(action: string, ...params: any[]) {
    try {
      const r = await invoke(action as any, { args: params });
      return tryParseJSON(r);
    } catch (e) {
      const snake = toSnake(action);
      if (snake !== action) {
        try {
          const r2 = await invoke(snake as any, { args: params });
          return tryParseJSON(r2);
        } catch (e2) {
          // swallow and fallback to electron below
        }
      }
    }
  }

  async emit(eventName: string, payload?: any) {
    return emit(eventName, payload);
  }

  async on(eventName: string, cb: (...args: any[]) => void) {
    const unlistenFn = await listen(eventName, (event: any) =>
      cb(event.payload)
    );
    return unlistenFn;
  }

  platform = "darwin";
  darkModeToggle = (theme?: "dark" | "light" | "system") => {
    console.log("Toggling dark mode:", theme);
  };
  async broadcast(eventName: string, ...args: any[]) {
    return emit(eventName, args);
  }
}

const _agent = new TauriAgent();

export const actions: Actions = new Proxy(
  {},
  {
    get(_obj, key: keyof Actions) {
      return (...params: any[]) => _agent.call(key as string, ...params);
    },
  }
) as Actions;

export const agent = _agent;
