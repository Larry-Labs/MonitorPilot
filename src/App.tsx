import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MonitorCard } from "./components/monitor-card";
import { MonitorCardSkeleton } from "./components/monitor-card-skeleton";
import { FeatureTips } from "./components/feature-tips";
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
    <div className="min-h-screen bg-background text-foreground flex flex-col">
      <header className="relative border-b border-border overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-r from-primary/5 via-primary/3 to-transparent dark:from-primary/10 dark:via-primary/5" />
        <div className="relative px-6 py-5">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-9 h-9 rounded-lg bg-primary/10 dark:bg-primary/20">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
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
            </div>
            <div>
              <h1 className="text-lg font-semibold tracking-tight">MonitorPilot</h1>
              <p className="text-xs text-muted-foreground mt-0.5">
                通过 DDC/CI 协议控制显示器输入源，告别物理按键
              </p>
            </div>
          </div>
        </div>
      </header>

      <main className="flex-1 p-6 space-y-4">
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
          </div>
        )}

        {!loading && monitors.length === 0 && !error && (
          <div className="flex flex-col items-center justify-center py-20 space-y-5">
            <div className="relative">
              <div className="absolute inset-0 rounded-full bg-muted/50 blur-xl scale-150" />
              <div className="relative rounded-full bg-muted p-5">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="36"
                  height="36"
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
            </div>
            <div className="text-center space-y-2">
              <p className="text-sm font-medium text-foreground">
                未检测到 DDC/CI 兼容显示器
              </p>
              <div className="text-xs text-muted-foreground max-w-sm leading-relaxed space-y-1">
                <p>请检查以下项目：</p>
                <ul className="text-left list-disc list-inside space-y-0.5">
                  <li>显示器已开启 DDC/CI 功能（OSD 菜单 → 设置）</li>
                  <li>DP 或 HDMI 线缆已正确连接</li>
                  <li>macOS 用户需安装 m1ddc（<code className="text-xs px-1 py-0.5 rounded bg-muted">brew install m1ddc</code>）</li>
                </ul>
              </div>
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

        {!loading && <FeatureTips />}
      </main>

      <footer className="border-t border-border px-6 py-3 text-center">
        <p className="text-[10px] text-muted-foreground/60">
          MonitorPilot v0.1.0 · DDC/CI Monitor Input Switcher
        </p>
      </footer>
    </div>
  );
}

export default App;
