//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";

import { ScrollArea } from "@/components/ui/ScrollArea";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store/widget";
import { ILogViewerWidget, ILogViewerWidgetOptions } from "@/types/widgets";
import { useAutoScroll } from "@/hooks/use-auto-scroll";

export function LogViewerBackstageWidget(props: ILogViewerWidget) {
  const { id, metadata: { wsUrl, scriptType, script } = {} } = props;

  const { appendLogViewerHistory } = useWidgetStore();

  const wsRef = React.useRef<WebSocket | null>(null);

  React.useEffect(() => {
    if (!wsUrl || !scriptType || !script) {
      return;
    }

    wsRef.current = new WebSocket(wsUrl);

    wsRef.current.onopen = () => {
      console.log("[LogViewerWidget] WebSocket connected!");
      wsRef.current?.send(JSON.stringify(script));
    };

    wsRef.current.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);

        if (msg.type === "stdout" || msg.type === "stderr") {
          const line = msg.data;
          appendLogViewerHistory(id, [line]);
        } else if (msg.type === "exit") {
          const code = msg.code;
          appendLogViewerHistory(id, [
            `Process exited with code ${code}. Closing...`,
          ]);

          wsRef.current?.close();
        } else if (msg.status === "fail") {
          appendLogViewerHistory(id, [
            `Error: ${msg.message || "Unknown error"}\n`,
          ]);
        } else {
          appendLogViewerHistory(id, [
            `Unknown message: ${JSON.stringify(msg)}`,
          ]);
        }
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
      } catch (err) {
        // If it's not JSON, output it directly as text.
        appendLogViewerHistory(id, [event.data]);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, wsUrl, scriptType, script]);

  return <></>;
}

export function LogViewerFrontStageWidget(props: {
  id: string;
  options?: ILogViewerWidgetOptions;
}) {
  const { id, options } = props;

  const [searchInput, setSearchInput] = React.useState("");
  const defferedSearchInput = React.useDeferredValue(searchInput);

  const { logViewerHistory, widgets } = useWidgetStore();

  const scrollAreaRef = React.useRef<HTMLDivElement>(null);

  const { mutate: mutateAutoScroll } = useAutoScroll(scrollAreaRef);

  const { t } = useTranslation();

  const logsMemo = React.useMemo(() => {
    return logViewerHistory[id]?.history || [];
  }, [logViewerHistory, id]);

  const currentWidget = React.useMemo(() => {
    return widgets.find((w) => w.id === id);
  }, [widgets, id]);

  React.useEffect(() => {
    mutateAutoScroll();
  }, [currentWidget?.display_type, mutateAutoScroll]);

  return (
    <div className="flex h-full w-full flex-col" id={id}>
      {!options?.disableSearch && (
        <div className="h-12 w-full flex items-center space-x-2 px-2">
          <Input
            placeholder="Search"
            className="w-full"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
          />
          <Button variant="outline" className="hidden">
            {t("action.search")}
          </Button>
        </div>
      )}
      <ScrollArea
        className={cn("h-full w-full p-2", {
          "h-[calc(100%-3rem)]": options?.disableSearch,
        })}
        viewportRef={scrollAreaRef}
      >
        <LogViewerLogItemList
          logs={logsMemo}
          search={defferedSearchInput}
          prefix={`${id}-${currentWidget?.display_type}`}
        />
      </ScrollArea>
    </div>
  );
}

const string2uuid = (str: string) => {
  // Create a deterministic hash from the string
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32-bit integer
  }

  // Use the hash to create a deterministic UUID-like string
  const hashStr = Math.abs(hash).toString(16).padStart(8, "0");
  return (
    `${hashStr}-${hashStr.slice(0, 4)}` +
    `-4${hashStr.slice(4, 7)}-${hashStr.slice(7, 11)}-${hashStr.slice(0, 12)}`
  );
};

export interface ILogViewerLogItemProps {
  id: string;
  extension?: string;
  file?: string;
  line?: number;
  host?: string;
  message: string;
}

const string2LogItem = (str: string): ILogViewerLogItemProps => {
  const regex = /^(\w+)@([^:]+):(\d+)\s+\[([^\]]+)\]\s+(.+)$/;
  const match = str.match(regex);
  const randomId = string2uuid(str + new Date().getTime());
  if (!match) {
    return {
      id: randomId,
      message: str,
    };
  }
  const [, extension, file, line, host, message] = match;
  return {
    id: randomId,
    extension,
    file,
    line: parseInt(line, 10),
    host,
    message,
  };
};

function LogViewerLogItem(props: ILogViewerLogItemProps & { search?: string }) {
  const { id, extension, file, line, host, message, search } = props;

  return (
    <div
      className={cn(
        "font-mono text-xs py-0.5",
        "hover:bg-gray-100 dark:hover:bg-gray-800"
      )}
      id={id}
    >
      {extension && (
        <>
          <span className="text-blue-500 dark:text-blue-400">{extension}</span>
          <span className="text-gray-500 dark:text-gray-400">@</span>
        </>
      )}
      {file && (
        <>
          <span className="text-emerald-600 dark:text-emerald-400">{file}</span>
          <span className="text-gray-500 dark:text-gray-400">:</span>
        </>
      )}
      {line && (
        <span className="text-amber-600 dark:text-amber-400">{line}</span>
      )}
      {host && (
        <>
          <span className="text-gray-500 dark:text-gray-400"> [</span>
          <span className="text-purple-600 dark:text-purple-400">{host}</span>
          <span className="text-gray-500 dark:text-gray-400">] </span>
        </>
      )}
      {search ? (
        <span className="whitespace-pre-wrap">
          {message.split(search).map((part, i, arr) => (
            <React.Fragment key={i}>
              {part}
              {i < arr.length - 1 && (
                <span className="bg-yellow-200 dark:bg-yellow-800">
                  {search}
                </span>
              )}
            </React.Fragment>
          ))}
        </span>
      ) : (
        <span className="whitespace-pre-wrap">{message}</span>
      )}
    </div>
  );
}

function LogViewerLogItemList(props: {
  logs: string[];
  search?: string;
  prefix?: string;
}) {
  const { logs: rawLogs, search, prefix } = props;

  const logsMemo = React.useMemo(() => {
    return rawLogs.map(string2LogItem);
  }, [rawLogs]);

  const filteredLogs = React.useMemo(() => {
    if (!search) {
      return logsMemo;
    }
    return logsMemo.filter((log) => log.message.includes(search));
  }, [logsMemo, search]);

  return (
    <>
      {filteredLogs.map((log) => (
        <LogViewerLogItem
          key={`${prefix}-${log.id}`}
          {...log}
          search={search}
        />
      ))}
    </>
  );
}
