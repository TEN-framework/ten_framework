//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

import { useListTenCloudStorePackages } from "@/api/services/extension";
import { SpinnerLoading } from "@/components/Status/Loading";
import { useWidgetStore } from "@/store/widget";
import { ExtensionList } from "@/components/Widget/ExtensionWidget/ExtensionList";
import { ExtensionSearch } from "@/components/Widget/ExtensionWidget/ExtensionSearch";
import { ExtensionDetails } from "@/components/Widget/ExtensionWidget/ExtensionDetails";
import { cn, compareVersions } from "@/lib/utils";

import type { TooltipContentProps } from "@radix-ui/react-tooltip";
import type { IListTenCloudStorePackage } from "@/types/extension";

export const ExtensionStoreWidget = (props: {
  className?: string;
  toolTipSide?: TooltipContentProps["side"];
}) => {
  const { className, toolTipSide } = props;

  const { t } = useTranslation();
  const { data, error, isLoading } = useListTenCloudStorePackages();
  const { extSearch, extFilter } = useWidgetStore();

  const deferredSearch = React.useDeferredValue(extSearch);
  const [filteredPackages, versions] = React.useMemo(() => {
    if (!data?.packages)
      return [[], new Map<string, IListTenCloudStorePackage[]>()];
    const versions = new Map<string, IListTenCloudStorePackage[]>();
    data.packages.forEach((item) => {
      if (versions.has(item.name)) {
        const version = versions.get(item.name);
        if (version) {
          version.push(item);
          version.sort((a, b) => compareVersions(b.version, a.version));
        } else {
          versions.set(item.name, [item]);
        }
      } else {
        versions.set(item.name, [item]);
      }
    });
    const filteredPackageNames = Array.from(versions.keys()).filter((name) => {
      return name.toLowerCase().includes(deferredSearch.toLowerCase());
    });
    const sortedFilteredPackageNames = filteredPackageNames.sort((a, b) => {
      if (extFilter.sort === "name") {
        return a.localeCompare(b);
      }
      if (extFilter.sort === "name-desc") {
        return b.localeCompare(a);
      }
      return 0;
    });
    const filteredPackages = sortedFilteredPackageNames
      .map((name) => {
        return versions.get(name)?.[0];
      })
      .filter((item) => item !== undefined);
    return [filteredPackages, versions];
  }, [data?.packages, deferredSearch, extFilter.sort]);

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
    <div className={cn("flex flex-col w-full h-full", className)}>
      <div>
        <ExtensionSearch />
        <div
          className={cn(
            "select-none cursor-default",
            "bg-slate-100/80 dark:bg-gray-900/80 px-2 py-1",
            "text-gray-500 dark:text-gray-400",
            "text-xs font-semibold",
            "flex items-center gap-2"
          )}
        >
          {filteredPackages.length === 0 && deferredSearch.trim() !== "" && (
            <p className="w-fit">{t("extensionStore.noMatchResult")}</p>
          )}

          {filteredPackages.length > 0 && deferredSearch.trim() !== "" && (
            <p className="w-fit">
              {t("extensionStore.matchResult", {
                count: filteredPackages.length,
              })}
            </p>
          )}

          {deferredSearch.trim() === "" && (
            <p className="ml-auto w-fit">
              {t("extensionStore.installedWithSum", {
                count: undefined,
                total: versions.size || undefined,
              })}
            </p>
          )}
        </div>
      </div>

      <ExtensionList
        items={filteredPackages}
        versions={versions}
        toolTipSide={toolTipSide}
      />
    </div>
  );
};

export const ExtensionWidget = (props: {
  className?: string;
  versions: IListTenCloudStorePackage[];
  name: string;
}) => {
  const { className, versions, name } = props;

  if (versions?.length === 0) {
    return null;
  }

  return (
    <ExtensionDetails versions={versions} name={name} className={className} />
  );
};
