//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation, Trans } from "react-i18next";

import Popup from "@/components/Popup/Popup";
import { cn } from "@/lib/utils";
import { TEN_FRAMEWORK_URL, TEN_FRAMEWORK_GITHUB_URL } from "@/constants";
import { PoweredByTenFramework } from "@/components/AppBar/AppBar";
import { useWidgetStore } from "@/store/widget";

export const ABOUT_POPUP_ID = "about-popup";

export const AboutPopup = () => {
  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();

  return (
    <Popup
      id={ABOUT_POPUP_ID}
      title={t("header.menu.about")}
      onClose={() => removeWidget(ABOUT_POPUP_ID)}
    >
      <div className="text-center">
        <p
          className={cn(
            "italic text-base mb-5",
            "font-['Segoe_UI',Tahoma,Geneva,Verdana,sans-serif]"
          )}
        >
          <Trans
            components={[
              <PoweredByTenFramework className="font-bold text-foreground" />,
            ]}
            t={t}
            i18nKey="header.poweredByTenFramework"
          />
        </p>
        <p className="my-1">
          {t("header.officialSite")}:&nbsp;
          <a
            href={TEN_FRAMEWORK_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 underline"
          >
            {TEN_FRAMEWORK_URL}
          </a>
        </p>
        <p className="my-1">
          {t("header.github")}:&nbsp;
          <a
            href={TEN_FRAMEWORK_GITHUB_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 underline"
          >
            {TEN_FRAMEWORK_GITHUB_URL}
          </a>
        </p>
      </div>
    </Popup>
  );
};
