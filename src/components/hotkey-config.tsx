import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { Separator } from "./ui/separator";

interface HotkeyBinding {
  monitor_index: number;
  input_value: number;
  shortcut: string;
}

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

interface AppConfig {
  input_names: Record<string, string>;
  hotkeys: HotkeyBinding[];
}

interface HotkeyConfigProps {
  monitors: MonitorInfo[];
}

export function HotkeyConfig({ monitors }: HotkeyConfigProps) {
  const [hotkeys, setHotkeys] = useState<HotkeyBinding[]>([]);
  const [recording, setRecording] = useState<string | null>(null);
  const [savedMsg, setSavedMsg] = useState<string | null>(null);

  useEffect(() => {
    invoke<AppConfig>("cmd_get_config").then((config) => {
      setHotkeys(config.hotkeys || []);
    });
  }, []);

  const getHotkey = (monitorIndex: number, inputValue: number): string => {
    const binding = hotkeys.find(
      (h) => h.monitor_index === monitorIndex && h.input_value === inputValue
    );
    return binding?.shortcut || "";
  };

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!recording) return;
      e.preventDefault();
      e.stopPropagation();

      const parts: string[] = [];
      if (e.ctrlKey || e.metaKey) parts.push("CmdOrCtrl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");

      const key = e.key;
      if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
        const normalizedKey = key.length === 1 ? key.toUpperCase() : key;
        parts.push(normalizedKey);

        if (parts.length >= 2) {
          const shortcut = parts.join("+");
          const [monitorIdx, inputVal] = recording.split("-").map(Number);

          setHotkeys((prev) => {
            const filtered = prev.filter(
              (h) =>
                !(
                  h.monitor_index === monitorIdx && h.input_value === inputVal
                )
            );
            return [...filtered, { monitor_index: monitorIdx, input_value: inputVal, shortcut }];
          });
          setRecording(null);
        }
      }
    },
    [recording]
  );

  useEffect(() => {
    if (recording) {
      window.addEventListener("keydown", handleKeyDown);
      return () => window.removeEventListener("keydown", handleKeyDown);
    }
  }, [recording, handleKeyDown]);

  const saveHotkeys = async () => {
    try {
      await invoke("cmd_save_hotkeys", { hotkeys });
      setSavedMsg("快捷键已保存");
      setTimeout(() => setSavedMsg(null), 2000);
    } catch (e) {
      setSavedMsg(`保存失败: ${e}`);
    }
  };

  const removeHotkey = (monitorIndex: number, inputValue: number) => {
    setHotkeys((prev) =>
      prev.filter(
        (h) => !(h.monitor_index === monitorIndex && h.input_value === inputValue)
      )
    );
  };

  if (monitors.length === 0) return null;

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base font-medium">全局快捷键</CardTitle>
          <div className="flex items-center gap-2">
            {savedMsg && (
              <span className="text-xs text-muted-foreground">{savedMsg}</span>
            )}
            <Button variant="default" size="sm" onClick={saveHotkeys}>
              保存
            </Button>
          </div>
        </div>
      </CardHeader>
      <Separator />
      <CardContent className="pt-4 space-y-4">
        {monitors.map((monitor) => (
          <div key={monitor.index} className="space-y-2">
            <p className="text-sm font-medium">{monitor.model}</p>
            <div className="space-y-1">
              {monitor.supported_inputs.map((input) => {
                const key = `${monitor.index}-${input.value}`;
                const shortcut = getHotkey(monitor.index, input.value);
                const isRecording = recording === key;

                return (
                  <div
                    key={key}
                    className="flex items-center justify-between py-1.5 px-2 rounded hover:bg-muted/50"
                  >
                    <span className="text-sm text-muted-foreground">
                      {input.name}
                    </span>
                    <div className="flex items-center gap-2">
                      {shortcut && !isRecording && (
                        <>
                          <Badge variant="outline" className="text-xs font-mono">
                            {shortcut}
                          </Badge>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
                            onClick={() => removeHotkey(monitor.index, input.value)}
                          >
                            ×
                          </Button>
                        </>
                      )}
                      <Button
                        variant={isRecording ? "destructive" : "outline"}
                        size="sm"
                        className="text-xs h-7"
                        onClick={() =>
                          setRecording(isRecording ? null : key)
                        }
                      >
                        {isRecording ? "按下快捷键..." : shortcut ? "修改" : "设置"}
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        ))}
        <p className="text-xs text-muted-foreground pt-2">
          点击"设置"后按下快捷键组合（如 Ctrl+Shift+1），保存后即可全局使用。
        </p>
      </CardContent>
    </Card>
  );
}
