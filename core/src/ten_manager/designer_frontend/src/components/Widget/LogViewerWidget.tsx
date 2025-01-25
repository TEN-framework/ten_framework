//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";

import { ScrollArea } from "@/components/ui/ScrollArea";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";

interface ILogViewerWidgetProps {
  id?: string;
  data?: {
    wsUrl?: string;
    baseDir?: string;
    scriptName?: string;
  };
}

export default function LogViewerWidget(props: ILogViewerWidgetProps) {
  const { id, data } = props;

  const [searchInput, setSearchInput] = React.useState("");
  const defferedSearchInput = React.useDeferredValue(searchInput);

  // Save dynamic logs.
  const [logs, setLogs] = React.useState<string[]>([]);
  const wsRef = React.useRef<WebSocket | null>(null);

  React.useEffect(() => {
    if (!data?.wsUrl) {
      return;
    }

    wsRef.current = new WebSocket(data.wsUrl);

    wsRef.current.onopen = () => {
      console.log("[LogViewerWidget] WebSocket connected!");

      // Immediately send the "run" command after establishing a successful
      // connection.
      const baseDir = data.baseDir || "";
      const name = data.scriptName || "";

      const runMsg = {
        type: "run",
        base_dir: baseDir,
        name: name,
      };
      wsRef.current?.send(JSON.stringify(runMsg));
    };

    wsRef.current.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);

        if (msg.type === "stdout" || msg.type === "stderr") {
          const line = msg.data;
          setLogs((prev) => [...prev, line]);
        } else if (msg.type === "exit") {
          const code = msg.code;
          setLogs((prev) => [
            ...prev,
            `Process exited with code ${code}. Closing...`,
          ]);

          wsRef.current?.close();
        } else if (msg.status === "fail") {
          setLogs((prev) => [
            ...prev,
            `Error: ${msg.message || "Unknown error"}\n`,
          ]);
        } else {
          setLogs((prev) => [
            ...prev,
            `Unknown message: ${JSON.stringify(msg)}`,
          ]);
        }
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
      } catch (err) {
        // If it's not JSON, output it directly as text.
        setLogs((prev) => [...prev, event.data]);
      }
    };

    wsRef.current.onerror = (err) => {
      console.error("[LogViewerWidget] WebSocket error:", err);
    };

    wsRef.current.onclose = () => {
      console.log("[LogViewerWidget] WebSocket closed!");
    };

    return () => {
      // Close the connection when the component is unmounted.
      wsRef.current?.close();
    };
  }, [data?.wsUrl, data?.baseDir, data?.scriptName]);

  const filteredLogs = React.useMemo(() => {
    if (!defferedSearchInput) {
      return logs;
    }
    return logs.filter((line) => line.includes(defferedSearchInput));
  }, [logs, defferedSearchInput]);

  return (
    <div className="flex h-full w-full flex-col" id={id}>
      <div className="h-12 w-full flex items-center space-x-2 px-2">
        <Input
          placeholder="Search"
          className="w-full"
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
        />
        <Button variant="outline" className="hidden">
          Search
        </Button>
      </div>
      <ScrollArea className="h-[calc(100%-3rem)] w-full">
        <div className="p-2 text-sm font-mono">
          {filteredLogs.map((line, idx) => (
            <div
              key={`log-line-${idx}`}
              className={cn("py-0.5 hover:bg-gray-100 dark:hover:bg-gray-800")}
            >
              {line}
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
