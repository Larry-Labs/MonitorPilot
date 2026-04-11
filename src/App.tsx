import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MonitorCard } from "./components/monitor-card";
import { TooltipProvider } from "./components/ui/tooltip";

interface InputSource {
  value: number;
  name: string;
}

interface MonitorInfo {
  index: number;
  model: string;
  current_input: number | null;
  current_input_name: string;
  supported_inputs: InputSource[];
}

interface MonitorListResult {
  monitors: MonitorInfo[];
  error: string | null;
}

function App() {
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [switching, setSwitching] = useState<string | null>(null);

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
  }, [refreshMonitors]);

  const handleSwitch = async (monitorIndex: number, inputValue: number, inputName: string) => {
    const key = `${monitorIndex}-${inputValue}`;
    setSwitching(key);
    try {
      await invoke("cmd_switch_input", {
        monitorIndex,
        inputValue,
      });
      await refreshMonitors();
    } catch (e) {
      setError(String(e));
    } finally {
      setSwitching(null);
    }
  };

  return (
    <TooltipProvider>
      <div className="min-h-screen bg-background text-foreground">
        <header className="border-b border-border px-6 py-4">
          <h1 className="text-xl font-semibold tracking-tight">MonitorPilot</h1>
          <p className="text-sm text-muted-foreground">显示器输入源切换</p>
        </header>

        <main className="p-6 space-y-4">
          {error && (
            <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-sm text-destructive">
              {error}
            </div>
          )}

          {!loading && monitors.length === 0 && !error && (
            <div className="text-center py-16 space-y-3">
              <div className="text-4xl">🖥️</div>
              <p className="text-muted-foreground">
                未检测到 DDC/CI 兼容显示器
              </p>
              <p className="text-xs text-muted-foreground max-w-md mx-auto">
                请确认显示器已开启 DDC/CI 功能（在显示器 OSD 菜单中设置），
                并且正确连接了 DP 或 HDMI 线缆。
              </p>
            </div>
          )}

          {monitors.map((monitor) => (
            <MonitorCard
              key={monitor.index}
              monitor={monitor}
              switching={switching}
              onSwitch={handleSwitch}
            />
          ))}
        </main>
      </div>
    </TooltipProvider>
  );
}

export default App;
