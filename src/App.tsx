import { useEffect, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MonitorCard } from "./components/monitor-card";
import { MonitorCardSkeleton } from "./components/monitor-card-skeleton";
import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert";
import { Button } from "./components/ui/button";
import type { MonitorInfo, MonitorListResult, AppConfig, SwitchResult } from "./types/monitor";

type ToastState = {
  type: "switching" | "success" | "warning" | "error";
  message: string;
} | null;

const TOAST_COLORS = {
  switching: "bg-primary/5 text-primary border-primary/15",
  success: "bg-emerald-500/5 text-emerald-600 dark:text-emerald-400 border-emerald-500/15",
  warning: "bg-amber-500/5 text-amber-600 dark:text-amber-400 border-amber-500/15",
  error: "bg-destructive/5 text-destructive border-destructive/15",
} as const;

function App() {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [switching, setSwitching] = useState<string | null>(null);
  const [customNames, setCustomNames] = useState<Record<string, string>>({});
  const [toast, setToast] = useState<ToastState>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const switchLock = useRef(false);
  const switchingRef = useRef<string | null>(null);
  const mountedRef = useRef(true);
  const lastMonitorCount = useRef(0);
  const customNamesRef = useRef(customNames);
  customNamesRef.current = customNames;

  const showToast = useCallback((state: NonNullable<ToastState>, duration?: number) => {
    if (toastTimer.current) clearTimeout(toastTimer.current);
    setToast(state);
    if (duration) {
      toastTimer.current = setTimeout(() => setToast(null), duration);
    }
  }, []);

  useEffect(() => {
    return () => {
      mountedRef.current = false;
      if (toastTimer.current) clearTimeout(toastTimer.current);
    };
  }, []);

  const refreshMonitors = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<MonitorListResult>("cmd_get_monitors");
      if (result.error) {
        setError(result.error);
      }
      setMonitors(result.monitors);
      monitorsSnapshotRef.current = result.monitors;
      monitorsJsonRef.current = JSON.stringify(result.monitors);
      lastMonitorCount.current = result.monitors.length;
      emptyPollCount.current = 0;
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const pollFailCount = useRef(0);
  const emptyPollCount = useRef(0);
  const monitorsJsonRef = useRef("");
  const revertCooldownUntil = useRef(0);
  const pollGeneration = useRef(0);
  const isPolling = useRef(false);
  const monitorsSnapshotRef = useRef<MonitorInfo[]>([]);
  const POLL_FAIL_THRESHOLD = 3;
  const EMPTY_POLL_THRESHOLD = 2;
  const TOAST_SHORT_MS = 2500;
  const TOAST_LONG_MS = 4000;

  const silentRefresh = useCallback(async () => {
    if (Date.now() < revertCooldownUntil.current) return;
    if (isPolling.current) return;
    isPolling.current = true;
    const gen = pollGeneration.current;
    try {
      const result = await invoke<MonitorListResult>("cmd_get_monitors");
      if (gen !== pollGeneration.current) return;

      if (result.monitors.length === 0 && lastMonitorCount.current > 0) {
        emptyPollCount.current += 1;
        if (emptyPollCount.current < EMPTY_POLL_THRESHOLD) {
          pollFailCount.current = 0;
          return;
        }
      } else {
        emptyPollCount.current = 0;
      }

      const newJson = JSON.stringify(result.monitors);
      if (newJson !== monitorsJsonRef.current) {
        if (!switchingRef.current) {
          setMonitors(prev => {
            const merged = result.monitors.map(newM => {
              const oldM = prev.find(m => m.index === newM.index);
              if (newM.current_input == null && oldM?.current_input != null) {
                return { ...newM, current_input: oldM.current_input, current_input_name: oldM.current_input_name };
              }
              return newM;
            });
            monitorsSnapshotRef.current = merged;
            monitorsJsonRef.current = JSON.stringify(merged);
            return merged;
          });
        }
      }
      lastMonitorCount.current = result.monitors.length;
      if (!switchingRef.current) {
        setError(result.error || null);
      }
      pollFailCount.current = 0;
    } catch (e) {
      if (gen !== pollGeneration.current) return;
      pollFailCount.current += 1;
      console.warn(`轮询检测失败 (${pollFailCount.current}/${POLL_FAIL_THRESHOLD}):`, e);
      if (pollFailCount.current >= POLL_FAIL_THRESHOLD) {
        showToast({ type: "warning", message: "显示器状态同步异常，数据可能不是最新的" }, TOAST_LONG_MS);
        pollFailCount.current = 0;
      }
    } finally {
      isPolling.current = false;
    }
  }, [showToast]);

  useEffect(() => {
    refreshMonitors();
    invoke<AppConfig>("cmd_get_config")
      .then((config) => {
        setCustomNames(config.input_names || {});
      })
      .catch((e) => {
        console.error("加载配置失败:", e);
      });
  }, [refreshMonitors]);

  useEffect(() => {
    let cancelled = false;
    const listeners: (() => void)[] = [];

    const subscribe = (event: string, handler: () => void) => {
      listen(event, handler).then(fn => {
        if (cancelled) {
          fn();
        } else {
          listeners.push(fn);
        }
      });
    };

    subscribe("display-changed", () => {
      console.log("[MonitorPilot] 收到显示器变化事件");
      if (!switchingRef.current) refreshMonitors();
    });

    subscribe("tray-switch-done", () => {
      console.log("[MonitorPilot] 收到托盘切换完成事件");
      silentRefresh();
    });

    return () => {
      cancelled = true;
      listeners.forEach(fn => fn());
    };
  }, [refreshMonitors, silentRefresh]);

  useEffect(() => {
    const POLL_INTERVAL = 3000;
    let timer: ReturnType<typeof setInterval>;

    const startPolling = () => {
      timer = setInterval(() => {
        if (!switchingRef.current) silentRefresh();
      }, POLL_INTERVAL);
    };

    const handleVisibility = () => {
      clearInterval(timer);
      if (!document.hidden) {
        if (!switchingRef.current) silentRefresh();
        startPolling();
      }
    };

    startPolling();
    document.addEventListener("visibilitychange", handleVisibility);

    return () => {
      clearInterval(timer);
      document.removeEventListener("visibilitychange", handleVisibility);
    };
  }, [silentRefresh]);

  const handleSwitch = useCallback(async (monitorIndex: number, inputValue: number) => {
    if (switchLock.current) {
      showToast({ type: "switching", message: "操作进行中，请稍候..." }, TOAST_SHORT_MS);
      return;
    }
    switchLock.current = true;

    const key = `${monitorIndex}-${inputValue}`;
    pollGeneration.current += 1;
    switchingRef.current = key;

    const previousMonitors = monitorsSnapshotRef.current;
    setSwitching(key);
    setMonitors(prev =>
      prev.map(m => {
        if (m.index !== monitorIndex) return m;
        const targetInput = m.supported_inputs.find(i => i.value === inputValue);
        return {
          ...m,
          current_input: inputValue,
          current_input_name: targetInput?.name ?? m.current_input_name,
        };
      })
    );
    monitorsJsonRef.current = "";
    showToast({ type: "switching", message: "正在切换输入源..." });
    await new Promise(r => setTimeout(r, 50));
    const switchStart = Date.now();
    try {
      const result = await invoke<SwitchResult>("cmd_switch_input", { monitorIndex, inputValue });
      if (!mountedRef.current) return;

      if (result.status === "warning") {
        if (result.actual_input != null) {
          setMonitors(prev => {
            const restored = prev.map(m =>
              m.index === monitorIndex
                ? { ...m, current_input: result.actual_input! }
                : m
            );
            monitorsSnapshotRef.current = restored;
            monitorsJsonRef.current = JSON.stringify(restored);
            return restored;
          });
        } else {
          setMonitors(previousMonitors);
          monitorsSnapshotRef.current = previousMonitors;
          monitorsJsonRef.current = JSON.stringify(previousMonitors);
        }
        revertCooldownUntil.current = Date.now() + 8000;
        showToast({ type: "warning", message: result.message }, TOAST_LONG_MS);
      } else {
        monitorsJsonRef.current = "";
        showToast({ type: "success", message: result.message }, TOAST_SHORT_MS);
      }
    } catch (e) {
      if (!mountedRef.current) return;
      setMonitors(previousMonitors);
      monitorsSnapshotRef.current = previousMonitors;
      monitorsJsonRef.current = JSON.stringify(previousMonitors);
      revertCooldownUntil.current = Date.now() + 8000;
      showToast({ type: "error", message: String(e) }, TOAST_LONG_MS);
    } finally {
      const elapsed = Date.now() - switchStart;
      const MIN_VISUAL_MS = 600;
      if (elapsed < MIN_VISUAL_MS) {
        await new Promise(r => setTimeout(r, MIN_VISUAL_MS - elapsed));
      }
      switchLock.current = false;
      switchingRef.current = null;
      if (mountedRef.current) setSwitching(null);
    }
  }, [showToast]);

  const handleRename = useCallback(async (key: string, name: string) => {
    const previous = { ...customNamesRef.current };
    const updated = { ...customNamesRef.current };
    if (name) {
      updated[key] = name;
    } else {
      delete updated[key];
    }
    setCustomNames(updated);

    try {
      const existing = await invoke<AppConfig>("cmd_get_config");
      await invoke("cmd_save_config", { config: { ...existing, input_names: updated } });
    } catch (e) {
      setCustomNames(previous);
      showToast({ type: "error", message: String(e) }, TOAST_LONG_MS);
    }
  }, [showToast]);

  const handleDdcError = useCallback((msg: string) => {
    showToast({ type: "error", message: msg }, TOAST_LONG_MS);
  }, [showToast]);

  return (
    <div className="h-screen bg-background text-foreground flex flex-col overflow-hidden">
      <header className="relative border-b border-border/60 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/8 via-primary/4 to-transparent" />
        <div className="absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2" />
        <div className="relative px-5 py-3.5">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary text-primary-foreground shadow-sm shadow-primary/25">
              <svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <rect width="20" height="14" x="2" y="3" rx="2" />
                <line x1="8" x2="16" y1="21" y2="21" />
                <line x1="12" x2="12" y1="17" y2="21" />
              </svg>
            </div>
            <div>
              <h1 className="text-lg font-bold tracking-tight">MonitorPilot</h1>
              <p className="text-xs text-muted-foreground">
                DDC/CI 显示器输入源切换 · 告别物理按键
              </p>
            </div>
          </div>
        </div>
      </header>

      <main className="flex-1 px-5 py-4 space-y-3 overflow-y-auto">
        {error && (
          <Alert variant="destructive">
            <svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" x2="12" y1="8" y2="12" />
              <line x1="12" x2="12.01" y1="16" y2="16" />
            </svg>
            <AlertTitle>检测异常</AlertTitle>
            <AlertDescription className="flex items-start justify-between gap-3">
              <span>{error}</span>
              <Button
                variant="outline"
                size="sm"
                className="shrink-0 h-7 text-xs border-destructive/30 hover:bg-destructive/10"
                onClick={refreshMonitors}
                disabled={loading}
              >
                重新检测
              </Button>
            </AlertDescription>
          </Alert>
        )}

        {loading && (
          <div className="space-y-4">
            <MonitorCardSkeleton />
          </div>
        )}

        {!loading && monitors.length === 0 && (
          <div className="flex flex-col items-center justify-center py-16 space-y-5">
            <div className="relative">
              <div className="absolute inset-0 rounded-full bg-primary/10 blur-2xl scale-[2]" />
              <div className="relative rounded-2xl bg-gradient-to-br from-primary/10 to-primary/5 p-6 border border-primary/10">
                <svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="text-primary/60">
                  <rect width="20" height="14" x="2" y="3" rx="2" />
                  <line x1="8" x2="16" y1="21" y2="21" />
                  <line x1="12" x2="12" y1="17" y2="21" />
                </svg>
              </div>
            </div>
            <div className="text-center space-y-2.5">
              <p className="text-sm font-semibold text-foreground">未检测到 DDC/CI 兼容显示器</p>
              <div className="text-xs text-muted-foreground max-w-xs leading-relaxed space-y-1.5">
                <p>请检查以下项目：</p>
                <ul className="text-left list-none space-y-1">
                  <li className="flex items-start gap-2">
                    <span className="text-primary mt-0.5">•</span>
                    <span>显示器已开启 DDC/CI（OSD 菜单 → 设置）</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-primary mt-0.5">•</span>
                    <span>视频线缆已正确连接（DP / HDMI / DVI / VGA / USB-C）</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-primary mt-0.5">•</span>
                    <span>外接显示器已连接（内置屏幕不支持 DDC/CI）</span>
                  </li>
                </ul>
              </div>
              <Button
                variant="outline"
                size="sm"
                className="mt-2 text-xs"
                onClick={refreshMonitors}
                disabled={loading}
              >
                重新检测
              </Button>
            </div>
          </div>
        )}

        {!loading && monitors.length > 0 && (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <p className="text-xs font-medium text-muted-foreground">
                检测到 {monitors.length} 台显示器
              </p>
            </div>
            {monitors.map((monitor) => (
              <MonitorCard
                key={monitor.index}
                monitor={monitor}
                switching={switching}
                customNames={customNames}
                onSwitch={handleSwitch}
                onRename={handleRename}
                onDdcError={handleDdcError}
              />
            ))}
          </div>
        )}
      </main>

      {toast && (
        <div
          role={toast.type === "error" ? "alert" : "status"}
          aria-live={toast.type === "error" ? "assertive" : "polite"}
          className={`mx-4 mb-2 px-3.5 py-2.5 rounded-lg text-xs font-medium flex items-center gap-2.5 border ${TOAST_COLORS[toast.type]}`}
        >
          {toast.type === "switching" && (
            <svg aria-hidden="true" className="animate-spin h-3.5 w-3.5 shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
          )}
          {toast.type === "success" && (
            <svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="shrink-0">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          )}
          {toast.type === "warning" && (
            <svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="shrink-0">
              <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" />
              <line x1="12" x2="12" y1="9" y2="13" />
              <line x1="12" x2="12.01" y1="17" y2="17" />
            </svg>
          )}
          {toast.type === "error" && (
            <svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="shrink-0">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" x2="9" y1="9" y2="15" />
              <line x1="9" x2="15" y1="9" y2="15" />
            </svg>
          )}
          <span className="leading-relaxed">{toast.message}</span>
        </div>
      )}

      <footer className="border-t border-border/40 px-5 py-2">
        <p className="text-[10px] text-muted-foreground/70 text-center">
          MonitorPilot v{__APP_VERSION__}
        </p>
      </footer>
    </div>
  );
}

export default App;
