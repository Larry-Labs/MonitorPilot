import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Separator } from "./ui/separator";

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

interface MonitorCardProps {
  monitor: MonitorInfo;
  switching: string | null;
  customNames: Record<string, string>;
  onSwitch: (monitorIndex: number, inputValue: number, inputName: string) => void;
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
        <p className="text-xs text-muted-foreground mb-3">切换输入源（双击名称可自定义）</p>
        <div className="grid grid-cols-3 gap-2">
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
                  autoFocus
                />
              );
            }

            return (
              <Button
                key={input.value}
                variant={isActive ? "default" : "outline"}
                size="sm"
                className="text-xs"
                disabled={isSwitching}
                onClick={() => {
                  if (!isActive) {
                    onSwitch(monitor.index, input.value, input.name);
                  }
                }}
                onDoubleClick={(e) => {
                  e.preventDefault();
                  startEditing(input);
                }}
              >
                {isSwitching ? "切换中..." : displayName}
              </Button>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
