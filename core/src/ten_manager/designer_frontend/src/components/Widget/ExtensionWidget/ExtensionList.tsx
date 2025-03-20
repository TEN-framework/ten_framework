//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { BlocksIcon, CheckIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { FixedSizeList as VirtualList } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";

import { Button } from "@/components/ui/Button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/Tooltip";
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store/widget";
import { EWidgetCategory, EWidgetDisplayType } from "@/types/widgets";
// eslint-disable-next-line max-len
import { ExtensionTooltipContent } from "@/components/Widget/ExtensionWidget/ExtensionDetails";
import {
  type IListTenCloudStorePackage,
  type IListTenLocalStorePackage,
  ETenPackageType,
  ITenPackage,
  ITenPackageLocal,
} from "@/types/extension";

import type { TooltipContentProps } from "@radix-ui/react-tooltip";

export const ExtensionList = (props: {
  items: (ITenPackage | ITenPackageLocal)[];
  versions: Map<string, IListTenCloudStorePackage[]>;
  className?: string;
  toolTipSide?: TooltipContentProps["side"];
  readOnly?: boolean;
}) => {
  const { items, versions, className, toolTipSide, readOnly } = props;

  const VirtualListItem = (props: {
    index: number;
    style: React.CSSProperties;
  }) => {
    const item = items[props.index];

    if (item._type === ETenPackageType.Local) {
      return (
        <AddonItem
          item={item}
          style={props.style}
          toolTipSide={toolTipSide}
          _type={item._type}
        />
      );
    }

    const targetVersions = versions.get(item.name);

    return (
      <ExtensionStoreItem
        item={item}
        style={props.style}
        toolTipSide={toolTipSide}
        versions={targetVersions}
        isInstalled={item.isInstalled}
        readOnly={readOnly}
      />
    );
  };

  return (
    <div className={cn("w-full h-full", className)}>
      <AutoSizer>
        {({ width, height }: { width: number; height: number }) => (
          <VirtualList
            width={width}
            height={height}
            itemCount={items.length}
            itemSize={52}
          >
            {VirtualListItem}
          </VirtualList>
        )}
      </AutoSizer>
    </div>
  );
};

export const ExtensionBaseItem = React.forwardRef<
  HTMLLIElement,
  {
    item: IListTenCloudStorePackage | IListTenLocalStorePackage;
    _type?: ETenPackageType;
    isInstalled?: boolean;
    className?: string;
    readOnly?: boolean;
  } & React.HTMLAttributes<HTMLLIElement>
>((props, ref) => {
  const { item, className, isInstalled, _type, readOnly, ...rest } = props;

  const { t } = useTranslation();

  return (
    <li
      ref={ref}
      className={cn(
        "px-1 py-2",
        "flex gap-2 w-full items-center max-w-full font-roboto h-fit",
        "hover:bg-gray-100 dark:hover:bg-gray-800",
        {
          "cursor-pointer": !!rest?.onClick && !readOnly,
        },
        className
      )}
      {...rest}
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
          {_type === ETenPackageType.Local && (
            <span
              className={cn(
                "text-gray-500 dark:text-gray-400",
                "text-xs font-normal"
              )}
            >
              {t("extensionStore.localAddonTip")}
            </span>
          )}
        </h3>
        <p className={cn("text-xs text-gray-500 dark:text-gray-400")}>
          {item.type}
        </p>
      </div>
      <div className="mt-auto flex flex-col items-end">
        {isInstalled ? (
          <>
            <Button
              variant="ghost"
              size="icon"
              disabled
              className={cn(
                "size-4 [&_svg]:size-3  cursor-pointer",
                "hover:bg-gray-200 dark:hover:bg-gray-700"
              )}
            >
              <CheckIcon className="" />
            </Button>
          </>
        ) : (
          <Button
            variant="secondary"
            size="sm"
            className={cn(
              "text-xs px-2 py-0.5 font-normal h-fit cursor-pointer",
              "shadow-none rounded-none",
              "hover:bg-gray-200 dark:hover:bg-gray-700"
            )}
            disabled={readOnly}
          >
            {t("extensionStore.install")}
          </Button>
        )}
      </div>
    </li>
  );
});
ExtensionBaseItem.displayName = "ExtensionBaseItem";

export const ExtensionStoreItem = (props: {
  item: IListTenCloudStorePackage;
  versions?: IListTenCloudStorePackage[];
  className?: string;
  style?: React.CSSProperties;
  toolTipSide?: TooltipContentProps["side"];
  isInstalled?: boolean;
  readOnly?: boolean;
}) => {
  const {
    item,
    versions,
    className,
    style,
    toolTipSide = "right",
    isInstalled,
    readOnly,
  } = props;

  const { appendWidgetIfNotExists } = useWidgetStore();

  const handleClick = () => {
    appendWidgetIfNotExists({
      id: `extension-${item.name}`,
      category: EWidgetCategory.Extension,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        name: item.name,
        versions: versions || [],
      },
    });
  };

  return (
    <TooltipProvider delayDuration={100}>
      <Tooltip>
        <TooltipTrigger asChild>
          <ExtensionBaseItem
            item={item}
            isInstalled={isInstalled}
            onClick={handleClick}
            className={className}
            style={style}
            readOnly={readOnly}
          />
        </TooltipTrigger>
        <TooltipContent
          side={toolTipSide}
          className={cn(
            "backdrop-blur-xs shadow-xl text-popover-foreground",
            "bg-slate-50/80 dark:bg-gray-900/90",
            "border border-slate-100/50 dark:border-gray-900/50",
            "ring-1 ring-slate-100/50 dark:ring-gray-900/50",
            "font-roboto"
          )}
        >
          <ExtensionTooltipContent item={item} versions={versions || []} />
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};

export const AddonItem = (props: {
  item: IListTenLocalStorePackage;
  style?: React.CSSProperties;
  toolTipSide?: TooltipContentProps["side"];
  _type?: ETenPackageType;
  readOnly?: boolean;
}) => {
  return (
    <ExtensionBaseItem
      item={props.item}
      isInstalled={true}
      style={props.style}
      _type={props._type}
      readOnly={props.readOnly}
    />
  );
};
