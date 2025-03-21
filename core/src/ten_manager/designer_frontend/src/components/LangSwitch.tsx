//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { LanguagesIcon, CheckIcon } from "lucide-react";

import { Button } from "@/components/ui/Button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";

export function LanguageToggle() {
  const { i18n, t } = useTranslation();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="bg-transparent">
          <LanguagesIcon className="h-[1.2rem] w-[1.2rem]" />
          <span className="sr-only">{t("header.language.title")}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => i18n.changeLanguage("en-US")}>
          <span>{t("header.language.enUS")}</span>
          {i18n.language === "en-US" && (
            <CheckIcon className="ml-auto h-4 w-4" />
          )}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => i18n.changeLanguage("zh-CN")}>
          <span>{t("header.language.zhCN")}</span>
          {i18n.language === "zh-CN" && (
            <CheckIcon className="ml-auto h-4 w-4" />
          )}
        </DropdownMenuItem>{" "}
        <DropdownMenuItem onClick={() => i18n.changeLanguage("zh-TW")}>
          <span>{t("header.language.zhTW")}</span>
          {i18n.language === "zh-TW" && (
            <CheckIcon className="ml-auto h-4 w-4" />
          )}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => i18n.changeLanguage("ja-JP")}>
          <span>{t("header.language.jaJP")}</span>
          {i18n.language === "ja-JP" && (
            <CheckIcon className="ml-auto h-4 w-4" />
          )}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
