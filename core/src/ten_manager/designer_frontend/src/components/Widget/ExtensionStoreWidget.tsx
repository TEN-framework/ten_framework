//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { toast } from "sonner";
import { BlocksIcon, CogIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useListTenCloudStorePackages } from "@/api/services/extension";
import { SpinnerLoading } from "@/components/Status/Loading";
import { ScrollArea } from "@/components/ui/ScrollArea";
import { Button } from "@/components/ui/Button";
import { Badge } from "@/components/ui/Badge";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/Tooltip";
import { Separator } from "@/components/ui/Separator";
import { cn } from "@/lib/utils";

import type { IListTenCloudStorePackage } from "@/types/extension";

export const ExtensionStoreWidget = (props: {
  className?: string;
  colWidth?: number;
}) => {
  const { className, colWidth = 340 } = props;
  const { data, error, isLoading } = useListTenCloudStorePackages();

  React.useEffect(() => {
    if (error) {
      toast.error(error.message, {
        description: error?.message,
      });
    }
  }, [error]);

  if (isLoading) {
    return <SpinnerLoading className="mx-auto" />;
  }

  return (
    <div className={cn("flex flex-col gap-2 w-full h-full", className)}>
      <ScrollArea className="h-full w-full">
        <div className="flex flex-col w-full h-full">
          {data?.packages?.map((item) => (
            <ExtensionStoreItem
              key={item.hash}
              item={item}
              style={{ width: `${colWidth}px` }}
              className="pr-2"
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  );
};

function ExtensionStoreItem(props: {
  item: IListTenCloudStorePackage;
  className?: string;
  style?: React.CSSProperties;
}) {
  const { item, className, style } = props;

  const { t } = useTranslation();

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <li
            className={cn(
              "px-1 py-2",
              "flex gap-2 w-full items-center max-w-full font-roboto h-fit",
              "hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer",
              className
            )}
            style={style}
          >
            <BlocksIcon className="size-8" />
            <div
              className={cn(
                "flex flex-col  justify-between",
                "w-full overflow-hidden text-sm"
              )}
            >
              <h3
                className={cn(
                  "font-semibold w-full overflow-hidden text-ellipsis",
                  "text-gray-900 dark:text-gray-100"
                )}
              >
                {item.name}
              </h3>
              <p className={cn("text-xs text-gray-500 dark:text-gray-400")}>
                {item.type}
              </p>
            </div>
            <div className="mt-auto flex flex-col items-end">
              {/* <Badge
                variant="secondary"
                className="text-xs px-2 py-0.5 whitespace-nowrap font-medium"
              >
                installed
              </Badge> */}
              <Button
                variant="ghost"
                size="icon"
                className={cn(
                  "size-4 [&_svg]:size-3  cursor-pointer",
                  "hover:bg-gray-200 dark:hover:bg-gray-700"
                )}
              >
                <CogIcon className="" />
              </Button>
            </div>
          </li>
        </TooltipTrigger>
        <TooltipContent
          side="right"
          className={cn(
            "backdrop-blur-xs shadow-xl text-popover-foreground",
            "bg-cyan-50/80 dark:bg-gray-900/90",
            "border border-cyan-100/50 dark:border-gray-900/50",
            "ring-1 ring-cyan-100/50 dark:ring-gray-900/50",
            "font-roboto"
          )}
        >
          <div className="flex flex-col gap-1">
            <p className="font-semibold text-lg">
              {item.name}
              <Badge
                variant="secondary"
                className={cn(
                  "text-xs px-2 py-0.5 whitespace-nowrap font-medium ",
                  "ml-2"
                )}
              >
                {item.version}
              </Badge>
            </p>
            <p
              className={cn(
                "font-roboto-condensed",
                "text-xs text-gray-500 dark:text-gray-400"
              )}
            >
              {item.type}
            </p>
            <Separator />
            {item.supports && (
              <>
                <div className="text-gray-500 dark:text-gray-400">
                  <p className="mb-1">{t("extensionStore.compatible")}</p>
                  <ul className="flex flex-wrap gap-1">
                    {item.supports?.map((support) => (
                      <Badge
                        key={`${item.name}-${support.os}-${support.arch}`}
                        variant="secondary"
                        className={cn(
                          "text-xs px-2 py-0.5 whitespace-nowrap font-medium"
                        )}
                      >
                        {support.os} {support.arch}
                      </Badge>
                    ))}
                  </ul>
                </div>
                <Separator />
              </>
            )}
            <div className="text-gray-500 dark:text-gray-400">
              <p className="mb-1">{t("extensionStore.dependencies")}</p>
              <ul className="flex flex-col gap-1 ml-2">
                {item.dependencies?.map((dependency) => (
                  <li
                    key={dependency.name}
                    className="flex items-center w-full justify-between"
                  >
                    <span className="font-semibold">{dependency.name}</span>
                    <span className="ml-1">{dependency.version}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
