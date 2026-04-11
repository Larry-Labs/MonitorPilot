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
    setError(null);
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
    const previous = { ...customNames };
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
      setCustomNames(previous);
      setError(String(e));
    }
  };

  return (
    <div className="h-screen bg-background text-foreground flex flex-col overflow-hidden">
      <header className="relative border-b border-border/60 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/8 via-primary/4 to-transparent" />
        <div className="absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2" />
        <div className="relative px-5 py-3.5">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary text-primary-foreground shadow-sm shadow-primary/25">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
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
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
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
          <div className="flex flex-col items-center justify-center py-16 space-y-5">
            <div className="relative">
              <div className="absolute inset-0 rounded-full bg-primary/10 blur-2xl scale-[2]" />
              <div className="relative rounded-2xl bg-gradient-to-br from-primary/10 to-primary/5 p-6 border border-primary/10">
                <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="text-primary/60">
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
                    <span>DP 或 HDMI 线缆已正确连接</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-primary mt-0.5">•</span>
                    <span>外接显示器已连接（内置屏幕不支持 DDC/CI）</span>
                  </li>
                </ul>
              </div>
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
              />
            ))}
          </div>
        )}
      </main>

      <footer className="border-t border-border/40 px-5 py-2">
        <p className="text-[10px] text-muted-foreground/70 text-center">
          MonitorPilot v{__APP_VERSION__}
        </p>
      </footer>
    </div>
  );
}

export default App;
