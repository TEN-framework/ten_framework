//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { BlocksIcon, HardDriveDownloadIcon, CheckIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/Button";
import { Badge, BadgeProps } from "@/components/ui/Badge";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { Separator } from "@/components/ui/Separator";
import { cn } from "@/lib/utils";

import type { IListTenCloudStorePackage } from "@/types/extension";

export const ExtensionTooltipContent = (props: {
  item: IListTenCloudStorePackage;
  versions: IListTenCloudStorePackage[];
}) => {
  const { item } = props;

  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-1">
      <div className="font-semibold text-lg">
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
      </div>
      <div
        className={cn(
          "font-roboto-condensed",
          "text-xs text-gray-500 dark:text-gray-400"
        )}
      >
        {item.type}
      </div>
      <Separator />
      {item.supports && (
        <>
          <div className="text-gray-500 dark:text-gray-400">
            <div className="mb-1">{t("extensionStore.compatible")}</div>
            <ExtensionEleSupports name={item.name} supports={item.supports} />
          </div>
          <Separator />
        </>
      )}
      <div className="text-gray-500 dark:text-gray-400">
        <div className="mb-1">{t("extensionStore.dependencies")}</div>
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
  );
};

export const ExtensionDetails = (props: {
  versions: IListTenCloudStorePackage[];
  name: string;
  className?: string;
}) => {
  const { versions, name, className } = props;
  const [selectedVersion, setSelectedVersion] = React.useState<string>(
    versions[0]?.version || ""
  );

  const defaultVersionMemo = React.useMemo(() => {
    return versions[0];
  }, [versions]);
  const selectedVersionItemMemo = React.useMemo(() => {
    return versions.find((version) => version.version === selectedVersion);
  }, [versions, selectedVersion]);

  const { t } = useTranslation();

  return (
    <div
      className={cn(
        "w-full h-full flex flex-col p-4 gap-2",
        "overflow-y-auto",
        className
      )}
    >
      <div className="flex items-center gap-2">
        <BlocksIcon className="size-20" />
        <div className="flex flex-col gap-1">
          <div className="font-semibold text-lg">{name}</div>
          <div
            className={cn(
              "font-roboto-condensed",
              "text-gray-500 dark:text-gray-400"
            )}
          >
            {defaultVersionMemo.type}
          </div>
          <div className="flex gap-2">
            <Button
              variant="secondary"
              size="sm"
              className={cn(
                "cursor-pointer rounded-none",
                "[&>svg]:size-3 px-2 py-0.5 h-fit",
                "text-xs font-normal"
              )}
            >
              <HardDriveDownloadIcon className="size-3" />
              <span>{t("extensionStore.install")}</span>
            </Button>
            <Button
              variant="secondary"
              size="sm"
              disabled
              className={cn(
                "cursor-pointer rounded-none",
                "[&>svg]:size-3 px-2 py-0.5 h-fit",
                "text-xs font-normal"
              )}
            >
              <CheckIcon className="size-3" />
              <span>{t("extensionStore.installed")}</span>
            </Button>
          </div>
        </div>
      </div>
      <Separator />
      <div className="flex items-center justify-between gap-2 w-full">
        <span className="font-semibold">{t("extensionStore.version")}</span>
        <Select
          defaultValue={versions[0]?.version}
          value={selectedVersion}
          onValueChange={setSelectedVersion}
        >
          <SelectTrigger className="w-fit min-w-48">
            <SelectValue placeholder="Select a version" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectLabel>{t("extensionStore.versionHistory")}</SelectLabel>
              {versions.map((version) => (
                <SelectItem key={version.version} value={version.version}>
                  {version.version === defaultVersionMemo.version
                    ? `${version.version}(${t("extensionStore.versionLatest")})`
                    : version.version}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
      <Separator />
      <div className="flex items-center justify-between gap-2 w-full">
        <span className="font-semibold">{t("extensionStore.hash")}</span>
        <ExtensionEleBadge className="max-w-20">
          {selectedVersionItemMemo?.hash?.slice(0, 8)}
        </ExtensionEleBadge>
      </div>
      <div className="flex items-start justify-between gap-2 w-full">
        <span className="font-semibold">
          {t("extensionStore.dependencies")}
        </span>
        <ul className="flex flex-col items-end gap-2">
          {selectedVersionItemMemo?.dependencies?.map((dependency) => (
            <li key={dependency.name}>
              <ExtensionEleBadge className="w-fit">
                <span className="font-semibold">{dependency.name}</span>
                <span className="ml-1">{dependency.version}</span>
              </ExtensionEleBadge>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
};

const ExtensionEleBadge = (props: BadgeProps) => {
  const { className, ...rest } = props;

  return (
    <Badge
      variant="secondary"
      className={cn(
        "text-xs px-2 py-0.5 whitespace-nowrap font-medium w-fit",
        className
      )}
      {...rest}
    />
  );
};

const ExtensionEleSupports = (props: {
  name: string;
  supports: IListTenCloudStorePackage["supports"];
  className?: string;
}) => {
  const { name, supports, className } = props;

  return (
    <ul className={cn("flex flex-wrap gap-1", className)}>
      {supports?.map((support) => (
        <ExtensionEleBadge key={`${name}-${support.os}-${support.arch}`}>
          {support.os} {support.arch}
        </ExtensionEleBadge>
      ))}
    </ul>
  );
};
