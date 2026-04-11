import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
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
  onSwitch: (monitorIndex: number, inputValue: number, inputName: string) => void;
}

export function MonitorCard({ monitor, switching, onSwitch }: MonitorCardProps) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base font-medium">
            {monitor.model}
          </CardTitle>
          <Badge variant="secondary" className="text-xs">
            {monitor.current_input_name}
          </Badge>
        </div>
      </CardHeader>
      <Separator />
      <CardContent className="pt-4">
        <p className="text-xs text-muted-foreground mb-3">切换输入源</p>
        <div className="grid grid-cols-3 gap-2">
          {monitor.supported_inputs.map((input) => {
            const isActive = monitor.current_input === input.value;
            const switchKey = `${monitor.index}-${input.value}`;
            const isSwitching = switching === switchKey;

            return (
              <Button
                key={input.value}
                variant={isActive ? "default" : "outline"}
                size="sm"
                className="text-xs"
                disabled={isActive || isSwitching}
                onClick={() => onSwitch(monitor.index, input.value, input.name)}
              >
                {isSwitching ? "切换中..." : input.name}
              </Button>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
