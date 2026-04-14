import { useState, useCallback, useRef, useEffect, memo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Slider } from "./ui/slider";
import { Button } from "./ui/button";
import type { MonitorInfo } from "../types/monitor";

interface DdcControlsProps {
  monitor: MonitorInfo;
  onError?: (message: string) => void;
}

const DEBOUNCE_MS = 300;
const POWER_ON = 1;
const POWER_STANDBY = 4;

function DdcSlider({
  label,
  icon,
  value,
  monitorIndex,
  command,
  onError,
}: {
  label: string;
  icon: React.ReactNode;
  value: number;
  monitorIndex: number;
  command: string;
  onError?: (message: string) => void;
}) {
  const [local, setLocal] = useState(value);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    if (!timerRef.current) setLocal(value);
  }, [value]);

  useEffect(() => {
    return () => { if (timerRef.current) clearTimeout(timerRef.current); };
  }, []);

  const handleChange = useCallback(
    (val: number | readonly number[]) => {
      const v = Array.isArray(val) ? val[0] : val;
      setLocal(v);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        timerRef.current = undefined;
        invoke(command, { monitorIndex, value: v }).catch((e) => {
          console.error(`${label}设置失败:`, e);
          onError?.(`${label}设置失败`);
        });
      }, DEBOUNCE_MS);
    },
    [monitorIndex, command, label, onError]
  );

  return (
    <div className="flex items-center gap-3">
      <div className="flex items-center gap-1.5 w-16 shrink-0 text-xs text-muted-foreground">
        {icon}
        <span>{label}</span>
      </div>
      <Slider
        value={[local]}
        onValueChange={handleChange}
        max={100}
        step={1}
        className="flex-1"
      />
      <span className="w-8 text-right text-xs tabular-nums text-muted-foreground">
        {local}
      </span>
    </div>
  );
}

function DdcIcon({ children }: { children: React.ReactNode }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
      fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      {children}
    </svg>
  );
}

const ICONS = {
  brightness: <DdcIcon><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" /></DdcIcon>,
  contrast: <DdcIcon><circle cx="12" cy="12" r="10" /><path d="M12 2a10 10 0 0 1 0 20z" fill="currentColor" /></DdcIcon>,
  volume: <DdcIcon><path d="M11 5 6 9H2v6h4l5 4zM15.54 8.46a5 5 0 0 1 0 7.07" /></DdcIcon>,
  power: <DdcIcon><path d="M12 2v10M18.36 6.64A9 9 0 1 1 5.64 6.64" /></DdcIcon>,
};

export const DdcControls = memo(function DdcControls({ monitor, onError }: DdcControlsProps) {
  const hasBrightness = monitor.brightness !== null;
  const hasContrast = monitor.contrast !== null;
  const hasVolume = monitor.volume !== null;
  const hasPower = monitor.power_mode !== null;
  const hasAny = hasBrightness || hasContrast || hasVolume || hasPower;

  if (!hasAny) return null;

  const isPowerOn = monitor.power_mode === POWER_ON;

  return (
    <div className="space-y-3 pt-2 border-t border-border/40">
      {hasBrightness && (
        <DdcSlider label="亮度" icon={ICONS.brightness} value={monitor.brightness!}
          monitorIndex={monitor.index} command="cmd_set_brightness" onError={onError} />
      )}
      {hasContrast && (
        <DdcSlider label="对比" icon={ICONS.contrast} value={monitor.contrast!}
          monitorIndex={monitor.index} command="cmd_set_contrast" onError={onError} />
      )}
      {hasVolume && (
        <DdcSlider label="音量" icon={ICONS.volume} value={monitor.volume!}
          monitorIndex={monitor.index} command="cmd_set_volume" onError={onError} />
      )}
      {hasPower && (
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5 w-16 shrink-0 text-xs text-muted-foreground">
            {ICONS.power}
            <span>电源</span>
          </div>
          <Button
            variant={isPowerOn ? "default" : "outline"}
            size="sm"
            className="h-7 text-xs px-3"
            onClick={() => {
              const mode = isPowerOn ? POWER_STANDBY : POWER_ON;
              invoke("cmd_set_power_mode", { monitorIndex: monitor.index, mode }).catch(
                (e) => {
                  console.error("电源模式设置失败:", e);
                  onError?.("电源模式设置失败");
                }
              );
            }}
          >
            {isPowerOn ? "开启" : "待机"}
          </Button>
        </div>
      )}
    </div>
  );
});
