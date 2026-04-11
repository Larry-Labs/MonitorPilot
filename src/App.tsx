import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MonitorCard } from "./components/monitor-card";
import { MonitorCardSkeleton } from "./components/monitor-card-skeleton";
import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert";
import type { MonitorInfo, MonitorListResult, AppConfig } from "./types/monitor";

function App() {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [switching, setSwitching] = useState<string | null>(null);
  const [customNames, setCustomNames] = useState<Record<string, string>>({});

  const refreshMonitors = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<MonitorListResult>("cmd_get_monitors");
      if (result.error) {
        setError(result.error);
      }
      setMonitors(result.monitors);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

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

  const handleSwitch = async (monitorIndex: number, inputValue: number) => {
    const key = `${monitorIndex}-${inputValue}`;
    setSwitching(key);
    try {
      await invoke("cmd_switch_input", { monitorIndex, inputValue });
      await refreshMonitors();
    } catch (e) {
      setError(String(e));
    } finally {
      setSwitching(null);
    }
  };

  const handleRename = async (key: string, name: string) => {
    const updated = { ...customNames };
    if (name) {
      updated[key] = name;
    } else {
      delete updated[key];
    }
    setCustomNames(updated);

    try {
      await invoke("cmd_save_config", { config: { input_names: updated } });
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="min-h-screen bg-background text-foreground">
        <header className="border-b border-border px-6 py-4">
          <div className="flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="text-primary"
            >
              <rect width="20" height="14" x="2" y="3" rx="2" />
              <line x1="8" x2="16" y1="21" y2="21" />
              <line x1="12" x2="12" y1="17" y2="21" />
            </svg>
            <div>
              <h1 className="text-lg font-semibold tracking-tight">MonitorPilot</h1>
              <p className="text-xs text-muted-foreground">显示器输入源切换</p>
            </div>
          </div>
        </header>

        <main className="p-6 space-y-4">
          {error && (
            <Alert variant="destructive">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="12" x2="12" y1="8" y2="12" />
                <line x1="12" x2="12.01" y1="16" y2="16" />
              </svg>
              <AlertTitle>检测异常</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {loading && (
            <div className="space-y-4">
              <MonitorCardSkeleton />
              <MonitorCardSkeleton />
            </div>
          )}

          {!loading && monitors.length === 0 && !error && (
            <div className="flex flex-col items-center justify-center py-16 space-y-4">
              <div className="rounded-full bg-muted p-4">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="32"
                  height="32"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  className="text-muted-foreground"
                >
                  <rect width="20" height="14" x="2" y="3" rx="2" />
                  <line x1="8" x2="16" y1="21" y2="21" />
                  <line x1="12" x2="12" y1="17" y2="21" />
                </svg>
              </div>
              <div className="text-center space-y-2">
                <p className="text-sm font-medium text-foreground">
                  未检测到 DDC/CI 兼容显示器
                </p>
                <p className="text-xs text-muted-foreground max-w-sm leading-relaxed">
                  请确认显示器已开启 DDC/CI 功能（在显示器 OSD 菜单中设置），
                  并且正确连接了 DP 或 HDMI 线缆。
                </p>
              </div>
            </div>
          )}

          {!loading &&
            monitors.map((monitor) => (
              <MonitorCard
                key={monitor.index}
                monitor={monitor}
                switching={switching}
                customNames={customNames}
                onSwitch={handleSwitch}
                onRename={handleRename}
              />
            ))}
        </main>
    </div>
  );
}

export default App;
