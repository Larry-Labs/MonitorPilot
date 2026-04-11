import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Separator } from "./ui/separator";
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

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base font-medium">
            {monitor.model}
          </CardTitle>
          <Badge variant="secondary" className="text-xs">
            {monitor.current_input != null
              ? getDisplayName(
                  monitor.supported_inputs.find(
                    (i) => i.value === monitor.current_input
                  ) || { value: monitor.current_input, name: monitor.current_input_name }
                )
              : monitor.current_input_name}
          </Badge>
        </div>
      </CardHeader>
      <Separator />
      <CardContent className="pt-4">
        <p className="text-xs text-muted-foreground mb-3">切换输入源</p>
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
                  className="h-8 text-xs"
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
                  className="w-full text-xs"
                  disabled={isSwitching}
                  onClick={() => {
                    if (!isActive) {
                      onSwitch(monitor.index, input.value);
                    }
                  }}
                >
                  {isSwitching ? "切换中..." : displayName}
                </Button>
                <button
                  className="absolute -top-1 -right-1 hidden group-hover:flex items-center justify-center w-4 h-4 rounded-full bg-muted border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                  onClick={(e) => {
                    e.stopPropagation();
                    startEditing(input);
                  }}
                  title="重命名"
                  aria-label={`编辑 ${displayName} 的名称`}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="8"
                    height="8"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
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
