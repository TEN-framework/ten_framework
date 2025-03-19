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
import {
  IListTenCloudStorePackage,
  ITenPackageLocal,
  ITenPackage,
  IListTenLocalStorePackage,
  ETenPackageType,
} from "@/types/extension";

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

  const [matched, versions, packagesMetadata] = React.useMemo(() => {
    const cloudExtNames = data?.packages?.map((item) => item.name) || [];
    const [localOnlyAddons, otherAddons] = addons.reduce(
      ([localOnly, other], addon) => {
        if (cloudExtNames.includes(addon.name)) {
          other.push(addon);
        } else {
          localOnly.push(addon);
        }
        return [localOnly, other];
      },
      [[], []] as [IListTenLocalStorePackage[], IListTenLocalStorePackage[]]
    );
    // todo: support os/arch
    const [installedPackages, uninstalledPackages] = (
      data?.packages || []
    ).reduce(
      ([installed, uninstalled], item) => {
        const packageName = item.name;
        const isInstalled = otherAddons.some(
          (addon) => addon.name === packageName
        );
        if (isInstalled) {
          installed.push(item);
        } else {
          uninstalled.push(item);
        }
        return [installed, uninstalled];
      },
      [[], []] as [IListTenCloudStorePackage[], IListTenCloudStorePackage[]]
    );
    const packagesMetadata = {
      localOnlyAddons: localOnlyAddons.map((item) => ({
        ...item,
        isInstalled: true,
        _type: ETenPackageType.Local,
      })) as ITenPackageLocal[],
      installedPackages: installedPackages.map((item) => ({
        ...item,
        isInstalled: true,
        _type: ETenPackageType.Default,
      })) as ITenPackage[],
      uninstalledPackages: uninstalledPackages.map((item) => ({
        ...item,
        isInstalled: false,
        _type: ETenPackageType.Default,
      })) as ITenPackage[],
      installedPackageNames: [
        ...new Set(installedPackages.map((item) => item.name)),
      ],
      uninstalledPackageNames: [
        ...new Set(uninstalledPackages.map((item) => item.name)),
      ],
    };
    const versions = new Map<string, IListTenCloudStorePackage[]>();
    data?.packages?.forEach((item) => {
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
    // --- filter ---
    // name matched
    const nameMatchedLocalOnlyAddons = packagesMetadata.localOnlyAddons.filter(
      (item) => {
        return item.name.toLowerCase().includes(deferredSearch.toLowerCase());
      }
    );
    const nameMatchedInstalledPackages =
      packagesMetadata.installedPackageNames.filter((item) => {
        return item.toLowerCase().includes(deferredSearch.toLowerCase());
      });
    const nameMatchedUninstalledPackages =
      packagesMetadata.uninstalledPackageNames.filter((item) => {
        return item.toLowerCase().includes(deferredSearch.toLowerCase());
      });
    // sort
    const sortedNameMatchedLocalOnlyAddons = nameMatchedLocalOnlyAddons.sort(
      (a, b) => {
        if (extFilter.sort === "name") {
          return a.name.localeCompare(b.name);
        }
        if (extFilter.sort === "name-desc") {
          return b.name.localeCompare(a.name);
        }
        return 0;
      }
    );
    const sortedNameMatchedInstalledPackageNames =
      nameMatchedInstalledPackages.sort((a, b) => {
        if (extFilter.sort === "name") {
          return a.localeCompare(b);
        }
        if (extFilter.sort === "name-desc") {
          return b.localeCompare(a);
        }
        return 0;
      });
    const sortedNameMatchedInstalledPackages: ITenPackage[] =
      sortedNameMatchedInstalledPackageNames.reduce<ITenPackage[]>(
        (acc, item) => {
          const version = versions.get(item);
          const target = version?.[0];
          if (!target) {
            return acc;
          }
          return [
            ...acc,
            {
              ...target,
              isInstalled: true,
              _type: ETenPackageType.Default,
            },
          ];
        },
        []
      );
    const sortedNameMatchedUninstalledPackageNames =
      nameMatchedUninstalledPackages.sort((a, b) => {
        if (extFilter.sort === "name") {
          return a.localeCompare(b);
        }
        if (extFilter.sort === "name-desc") {
          return b.localeCompare(a);
        }
        return 0;
      });
    const sortedNameMatchedUninstalledPackages =
      sortedNameMatchedUninstalledPackageNames.reduce<ITenPackage[]>(
        (acc, item) => {
          const version = versions.get(item);
          const target = version?.[0];
          if (!target) {
            return acc;
          }
          return [
            ...acc,
            {
              ...target,
              isInstalled: false,
              _type: ETenPackageType.Default,
            },
          ];
        },
        []
      );
    const matchedMetadata = {
      localOnlyAddons: sortedNameMatchedLocalOnlyAddons,
      installedPackageNames: sortedNameMatchedInstalledPackageNames,
      uninstalledPackageNames: sortedNameMatchedUninstalledPackageNames,
    };
    const matched = [
      ...(extFilter.showInstalled ? matchedMetadata.localOnlyAddons : []),
      ...(extFilter.showInstalled ? sortedNameMatchedInstalledPackages : []),
      ...(extFilter.showUninstalled
        ? sortedNameMatchedUninstalledPackages
        : []),
    ];

    return [matched, versions, packagesMetadata];
  }, [
    data?.packages,
    addons,
    deferredSearch,
    extFilter.sort,
    extFilter.showUninstalled,
    extFilter.showInstalled,
  ]);

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
          {matched.length === 0 && deferredSearch.trim() !== "" && (
            <p className="w-fit">{t("extensionStore.noMatchResult")}</p>
          )}

          {matched.length > 0 && deferredSearch.trim() !== "" && (
            <p className="w-fit">
              {t("extensionStore.matchResult", {
                count: matched.length,
              })}
            </p>
          )}

          {deferredSearch.trim() === "" &&
            extFilter.showInstalled &&
            extFilter.showUninstalled && (
              <p className="ml-auto w-fit">
                {t("extensionStore.installedWithSum", {
                  count:
                    packagesMetadata.localOnlyAddons.length +
                    packagesMetadata.installedPackageNames.length,
                  total:
                    packagesMetadata.localOnlyAddons.length +
                    packagesMetadata.installedPackageNames.length +
                    packagesMetadata.uninstalledPackageNames.length,
                })}
              </p>
            )}
        </div>
      </div>

      <ExtensionList
        items={matched}
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
