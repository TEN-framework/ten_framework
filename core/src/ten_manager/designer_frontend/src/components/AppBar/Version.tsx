//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { ArrowUpIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Badge } from "@/components/ui/Badge";
import { useVersion, useCheckUpdate } from "@/api/services/common";
import { cn } from "@/lib/utils";
import { TEN_FRAMEWORK_RELEASE_URL } from "@/constants";

export function Version() {
  const { version } = useVersion();
  const { data: updateData } = useCheckUpdate();

  const { t } = useTranslation("header");

  return (
    <Badge variant="secondary" className="relative gap-2">
      <span className="uppercase">{t("ten")}</span>
      <a
        href={updateData?.release_page || TEN_FRAMEWORK_RELEASE_URL}
        target="_blank"
        className="inline-flex items-center gap-1"
      >
        <span className="uppercase"> {version}</span>

        {updateData?.update_available && (
          <ArrowUpIcon
            className={cn(
              "size-3 stroke-3 ",
              "text-emerald-500 animate-bounce"
            )}
          />
        )}
      </a>
    </Badge>
  );
}
