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
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store/widget";
import { ExtensionSearch } from "@/components/Widget/ExtensionStoreWidget/ExtensionSearch";
import { ExtensionList } from "@/components/Widget/ExtensionStoreWidget/ExtensionList";

export const ExtensionStoreWidget = (props: { className?: string }) => {
  const { className } = props;

  const { t } = useTranslation();
  const { data, error, isLoading } = useListTenCloudStorePackages();
  const { extSearch, extFilter } = useWidgetStore();

  const deferredSearch = React.useDeferredValue(extSearch);
  const filteredPackages = React.useMemo(() => {
    if (!data?.packages) return [];
    const filtered = data.packages.filter((item) => {
      return item.name.toLowerCase().includes(deferredSearch.toLowerCase());
    });
    const sorted = filtered.sort((a, b) => {
      if (extFilter.sort === "name") {
        return a.name.localeCompare(b.name);
      }
      if (extFilter.sort === "name-desc") {
        return b.name.localeCompare(a.name);
      }
      return 0;
    });
    return sorted;
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
            "bg-gray-100 dark:bg-gray-800 px-2 py-1",
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
                total: data?.totalSize || undefined,
              })}
            </p>
          )}
        </div>
      </div>

      <ExtensionList items={filteredPackages} />
    </div>
  );
};
