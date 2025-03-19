//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { LayoutGridIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useApps } from "@/api/services/apps";
import {
  EWidgetDisplayType,
  EDefaultWidgetType,
  EWidgetCategory,
} from "@/types/widgets";
import { APPS_MANAGER_POPUP_ID } from "@/constants/widgets";
import { useWidgetStore } from "@/store";

export default function StatusBar(props: { className?: string }) {
  const { className } = props;

  return (
    <footer
      className={cn(
        "flex justify-between items-center text-xs select-none",
        "h-5 w-full",
        "fixed bottom-0 left-0 right-0",
        "bg-background/80 backdrop-blur-xs",
        "border-t border-[#e5e7eb] dark:border-[#374151]",
        "select-none",
        className
      )}
    >
      <div className="flex w-full h-full">
        <StatusApps />
      </div>
    </footer>
  );
}

const StatusApps = () => {
  const { t } = useTranslation();
  const { data, error, isLoading } = useApps();
  const { appendWidgetIfNotExists } = useWidgetStore();

  const openAppsManagerPopup = () => {
    appendWidgetIfNotExists({
      id: APPS_MANAGER_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.AppsManager,
      },
    });
  };

  React.useEffect(() => {
    if (error) {
      toast.error(t("statusBar.appsError"));
    }
  }, [error, t]);

  if (isLoading || !data) {
    return null;
  }

  return (
    <Button
      variant="ghost"
      size="status"
      className=""
      onClick={openAppsManagerPopup}
    >
      <LayoutGridIcon className="size-3" />
      <span className="">
        {t("statusBar.appsAllListedWithCount", {
          count: data.base_dirs.length,
        })}
      </span>
    </Button>
  );
};
