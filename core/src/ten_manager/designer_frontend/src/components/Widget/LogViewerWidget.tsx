/* eslint-disable max-len */
import * as React from "react";

import { ScrollArea } from "@/components/ui/ScrollArea";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";

const MOCK_LOG_VIEWER_DATA = [
  "en_app_1@main.py:17 [localhost] Extension loaded.",
  "en_app_1@config.py:25 [localhost] Loading configuration file...",
  "en_app_1@config.py:42 [localhost] Configuration loaded successfully.",
  "en_app_2@server.py:89 [remote] Server starting on port 8080...",
  "en_app_2@server.py:91 [remote] Server started successfully.",
  "en_app_1@database.py:156 [localhost] Connecting to database...",
  "en_app_1@database.py:172 [localhost] Database connection established with parameters: host=localhost, port=5432, database=production_db, max_connections=100, idle_timeout=300, connection_timeout=60.",
  "en_app_3@worker.py:45 [worker-1] Worker process initialized.",
  "en_app_2@routes.py:78 [remote] Registering API endpoints...",
  "en_app_2@routes.py:92 [remote] Successfully registered the following API endpoints: /api/v1/users, /api/v1/posts, /api/v1/comments, /api/v1/auth/login, /api/v1/auth/logout, /api/v1/auth/refresh, /api/v1/admin/users, /api/v1/admin/settings",
  "en_app_1@auth.py:134 [localhost] Initializing authentication service...",
  "en_app_1@auth.py:156 [localhost] Authentication service ready with providers: OAuth2, JWT, Basic Auth, API Key. Supported algorithms: HS256, RS256, ES256. Token expiration: 3600s",
  "en_app_3@worker.py:67 [worker-1] Processing job queue...",
  "en_app_2@middleware.py:45 [remote] Loading middleware components...",
  "en_app_2@middleware.py:89 [remote] Middleware initialization complete. Active middleware chain: RateLimiter -> Authentication -> Compression -> Caching -> Logging -> CORS -> ErrorHandler",
  "en_app_1@cache.py:23 [localhost] Cache service starting...",
  "en_app_1@cache.py:45 [localhost] Cache service ready with configuration: engine=redis, max_memory=2GB, eviction_policy=LRU, persistence=enabled, backup_interval=1h",
  "en_app_3@tasks.py:112 [worker-1] Scheduled tasks loaded.",
  "en_app_2@websocket.py:67 [remote] WebSocket server initialized with configuration: max_connections=1000, heartbeat_interval=30s, message_queue_size=5000, ssl_enabled=true",
  "en_app_1@events.py:89 [localhost] Event dispatcher ready.",
  "en_app_3@worker.py:89 [worker-1] Processing task ID: 12345",
  "en_app_2@api.py:234 [remote] Handling request: GET /api/v1/users",
  "en_app_1@models.py:167 [localhost] Database migration detected. Executing migration scripts: 001_initial.sql, 002_add_user_preferences.sql, 003_modify_timestamps.sql, 004_add_indexes.sql, 005_update_constraints.sql",
  "en_app_1@models.py:189 [localhost] Database models synced.",
  "en_app_3@worker.py:92 [worker-1] Task 12345 completed successfully. Processed 1500 records, updated indexes, and generated report. Time taken: 45.23s, Memory usage: 256MB, CPU usage: 78%",
  "en_app_2@api.py:256 [remote] Request completed: 200 OK",
  "en_app_1@scheduler.py:78 [localhost] Scheduling background jobs...",
  "en_app_1@scheduler.py:92 [localhost] Background jobs scheduled with the following cron expressions: cleanup_tasks='0 0 * * *', update_indexes='*/15 * * * *', backup_database='0 2 * * *', send_reports='0 8 * * 1-5'",
  "en_app_3@worker.py:95 [worker-1] Processing task ID: 12346",
  "en_app_2@auth.py:145 [remote] User authentication successful: user123",
  "en_app_1@logger.py:56 [localhost] Rotating log files...",
  "en_app_1@logger.py:78 [localhost] Log rotation complete. Archived files: app.log.1, app.log.2, app.log.3, app.log.4, app.log.5. Total size: 1.2GB",
  // ... continuing with similar pattern for remaining lines to reach 100
  "en_app_2@api.py:789 [remote] Performance metrics for the last hour: Requests=15000, Average Response Time=45ms, Error Rate=0.02%, Cache Hit Ratio=89%, Active Users=1250, Peak Memory Usage=3.8GB",
  "en_app_1@system.py:456 [localhost] System health report: CPU Usage=65%, Memory Usage=78%, Disk Usage=52%, Network I/O=125MB/s, Active Connections=850, Thread Pool Usage=45%, Database Connections=85/100",
  "en_app_3@worker.py:234 [worker-1] Batch processing complete. Processed Items: 25000, Failed Items: 12, Retry Queue: 8, Processing Time: 12m 34s, Average Processing Rate: 2083 items/minute",
  "en_app_2@security.py:567 [remote] Security audit log: Failed login attempts=23, Blocked IPs=5, SQL Injection Attempts=3, XSS Attempts=7, Rate Limit Exceeded=156, Invalid API Keys=12, Suspicious User Agents=8",
];

export default function LogViewerWidget(props: { id: string }) {
  const { id } = props;

  const [searchInput, setSearchInput] = React.useState("");
  const defferedSearchInput = React.useDeferredValue(searchInput);

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
        <div className="p-2 text-sm">
          <LogViewerLogItemList
            logs={MOCK_LOG_VIEWER_DATA}
            search={defferedSearchInput}
          />
        </div>
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

const string2LogItem = (str: string): ILogViewerLogItemProps => {
  const regex = /^(\w+)@([^:]+):(\d+)\s+\[([^\]]+)\]\s+(.+)$/;
  const match = str.match(regex);
  if (!match) {
    throw new Error("Invalid log format");
  }
  const [, extension, file, line, host, message] = match;
  return {
    id: string2uuid(str),
    extension,
    file,
    line: parseInt(line, 10),
    host,
    message,
  };
};

export interface ILogViewerLogItemProps {
  id: string;
  extension: string;
  file: string;
  line: number;
  host: string;
  message: string;
}

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
      <span className="text-blue-500 dark:text-blue-400">{extension}</span>
      <span className="text-gray-500 dark:text-gray-400">@</span>
      <span className="text-emerald-600 dark:text-emerald-400">{file}</span>
      <span className="text-gray-500 dark:text-gray-400">:</span>
      <span className="text-amber-600 dark:text-amber-400">{line}</span>
      <span className="text-gray-500 dark:text-gray-400"> [</span>
      <span className="text-purple-600 dark:text-purple-400">{host}</span>
      <span className="text-gray-500 dark:text-gray-400">] </span>
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

function LogViewerLogItemList(props: { logs: string[]; search?: string }) {
  const { logs: rawLogs, search } = props;

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
        <LogViewerLogItem key={log.id} {...log} search={search} />
      ))}
    </>
  );
}
