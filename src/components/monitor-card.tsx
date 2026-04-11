import { useState } from "react";
import { Card, CardContent, CardHeader } from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import type { MonitorInfo, InputSource } from "../types/monitor";

interface MonitorCardProps {
  monitor: MonitorInfo;
  switching: string | null;
  customNames: Record<string, string>;
  onSwitch: (monitorIndex: number, inputValue: number) => void;
  onRename: (key: string, name: string) => void;
}

export function MonitorCard({ monitor, switching, customNames, onSwitch, onRename }: MonitorCardProps) {
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");

  const getDisplayName = (input: InputSource) => {
    const key = `${monitor.index}-${input.value}`;
    return customNames[key] || input.name;
  };

  const startEditing = (input: InputSource) => {
    const key = `${monitor.index}-${input.value}`;
    setEditingKey(key);
    setEditValue(customNames[key] || "");
  };

  const finishEditing = () => {
    if (editingKey) {
      onRename(editingKey, editValue.trim());
      setEditingKey(null);
      setEditValue("");
    }
  };

  const activeInput = monitor.supported_inputs.find(
    (i) => i.value === monitor.current_input
  );
  const activeDisplayName = activeInput
    ? getDisplayName(activeInput)
    : monitor.current_input_name;

  return (
    <Card className="overflow-hidden transition-all duration-200 hover:shadow-lg hover:shadow-primary/5 border-border/60">
      <CardHeader className="pb-3 pt-4 px-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="relative flex items-center justify-center w-9 h-9 rounded-lg bg-gradient-to-br from-primary/15 to-primary/5 border border-primary/10">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-primary">
                <rect width="20" height="14" x="2" y="3" rx="2" />
                <line x1="8" x2="16" y1="21" y2="21" />
                <line x1="12" x2="12" y1="17" y2="21" />
              </svg>
            </div>
            <div>
              <p className="text-sm font-semibold leading-tight">{monitor.model}</p>
              <p className="text-[10px] text-muted-foreground mt-0.5">当前输入: {activeDisplayName}</p>
            </div>
          </div>
          <Badge className="text-[11px] font-medium bg-primary/10 text-primary hover:bg-primary/15 border-primary/15 gap-1.5">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            {activeDisplayName}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4">
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {monitor.supported_inputs.map((input) => {
            const isActive = monitor.current_input === input.value;
            const switchKey = `${monitor.index}-${input.value}`;
            const isSwitching = switching === switchKey;
            const isEditing = editingKey === switchKey;
            const displayName = getDisplayName(input);

            if (isEditing) {
              return (
                <Input
                  key={input.value}
                  value={editValue}
                  onChange={(e) => setEditValue(e.target.value)}
                  onBlur={finishEditing}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") finishEditing();
                    if (e.key === "Escape") {
                      setEditingKey(null);
                      setEditValue("");
                    }
                  }}
                  placeholder={input.name}
                  className="h-10 text-xs border-primary/30 focus-visible:ring-primary/30"
                  aria-label={`重命名 ${input.name}`}
                  autoFocus
                />
              );
            }

            return (
              <div key={input.value} className="relative group">
                <Button
                  variant={isActive ? "default" : "outline"}
                  size="sm"
                  className={`w-full text-xs h-10 transition-all duration-200 font-medium ${
                    isActive
                      ? "shadow-md shadow-primary/20"
                      : "hover:border-primary/40 hover:bg-primary/5 hover:text-primary dark:hover:bg-primary/10"
                  }`}
                  disabled={isSwitching}
                  onClick={() => {
                    if (!isActive) {
                      onSwitch(monitor.index, input.value);
                    }
                  }}
                >
                  {isSwitching ? (
                    <span className="flex items-center gap-1.5">
                      <svg className="animate-spin h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                      </svg>
                      切换中
                    </span>
                  ) : isActive ? (
                    <span className="flex items-center gap-1.5">
                      <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                      {displayName}
                      <span className="text-[9px] opacity-75 font-normal">当前</span>
                    </span>
                  ) : (
                    displayName
                  )}
                </Button>
                <button
                  className="absolute -top-1.5 -right-1.5 hidden group-hover:flex items-center justify-center w-5 h-5 rounded-full bg-background border border-border shadow-sm text-muted-foreground hover:text-primary hover:border-primary/40 transition-all duration-150"
                  onClick={(e) => {
                    e.stopPropagation();
                    startEditing(input);
                  }}
                  title="重命名"
                  aria-label={`编辑 ${displayName} 的名称`}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" />
                  </svg>
                </button>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
