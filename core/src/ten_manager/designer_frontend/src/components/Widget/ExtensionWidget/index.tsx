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
import { retrieveAddons } from "@/api/services/addons";
import { useEnv } from "@/api/services/common";
import { SpinnerLoading } from "@/components/Status/Loading";
import { useWidgetStore, useAppStore } from "@/store";
// eslint-disable-next-line max-len
import { ExtensionList } from "@/components/Widget/ExtensionWidget/ExtensionList";
// eslint-disable-next-line max-len
import { ExtensionSearch } from "@/components/Widget/ExtensionWidget/ExtensionSearch";
// eslint-disable-next-line max-len
import { ExtensionDetails } from "@/components/Widget/ExtensionWidget/ExtensionDetails";
import { cn, compareVersions } from "@/lib/utils";

import type { TooltipContentProps } from "@radix-ui/react-tooltip";
import type { IListTenCloudStorePackage } from "@/types/extension";

export const ExtensionStoreWidget = (props: {
  className?: string;
  toolTipSide?: TooltipContentProps["side"];
}) => {
  const { className, toolTipSide } = props;
  const [isFetchingAddons, setIsFetchingAddons] = React.useState(false);

  const { t } = useTranslation();
  const { data, error, isLoading } = useListTenCloudStorePackages();
  const { data: envData, error: envError, isLoading: isLoadingEnv } = useEnv();
  const { extSearch, extFilter } = useWidgetStore();
  const { addons, setAddons } = useAppStore();

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

  const filteredAddons = React.useMemo(() => {
    const filteredAddons = addons.filter((item) => {
      return item.name.toLowerCase().includes(deferredSearch.toLowerCase());
    });
    const sortedFilteredAddons = filteredAddons.sort((a, b) => {
      if (extFilter.sort === "name") {
        return a.name.localeCompare(b.name);
      }
      if (extFilter.sort === "name-desc") {
        return b.name.localeCompare(a.name);
      }
      return 0;
    });
    return sortedFilteredAddons;
  }, [addons, deferredSearch, extFilter.sort]);

  React.useEffect(() => {
    const fetchAddons = async () => {
      try {
        setIsFetchingAddons(true);
        const res = await retrieveAddons({});
        setAddons(res);
      } catch (error) {
        console.error("Failed to fetch addons", error);
        toast.error("Failed to fetch addons", {
          description: error instanceof Error ? error.message : String(error),
        });
      } finally {
        setIsFetchingAddons(false);
      }
    };

    fetchAddons();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  React.useEffect(() => {
    if (error) {
      toast.error(error.message, {
        description: error?.message,
      });
    }
    if (envError) {
      toast.error(envError.message, {
        description: envError?.message,
      });
    }
  }, [error, envError]);

  if (isLoading || isFetchingAddons || isLoadingEnv) {
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
          {[...filteredPackages, ...filteredAddons].length === 0 &&
            deferredSearch.trim() !== "" && (
              <p className="w-fit">{t("extensionStore.noMatchResult")}</p>
            )}

          {[...filteredPackages, ...filteredAddons].length > 0 &&
            deferredSearch.trim() !== "" && (
              <p className="w-fit">
                {t("extensionStore.matchResult", {
                  count: filteredPackages.length + filteredAddons.length,
                })}
              </p>
            )}

          {deferredSearch.trim() === "" && (
            <p className="ml-auto w-fit">
              {t("extensionStore.installedWithSum", {
                count: filteredAddons.length,
                total: versions.size + (addons?.length || 0) || undefined,
              })}
            </p>
          )}
        </div>
      </div>

      <ExtensionList
        addons={filteredAddons}
        items={filteredPackages}
        versions={versions}
        toolTipSide={toolTipSide}
        defaultOsArch={envData}
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
