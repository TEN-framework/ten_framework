//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React from "react";
import { useTranslation } from "react-i18next";

import Popup from "@/components/Popup/Popup";
import { cn } from "@/lib/utils";
import { TEN_FRAMEWORK_URL, TEN_FRAMEWORK_GITHUB_URL } from "@/constants";

interface AboutPopupProps {
  onClose: () => void;
}

const AboutPopup: React.FC<AboutPopupProps> = ({ onClose }) => {
  const { t } = useTranslation();

  return (
    <Popup title={t("header.menu.about")} onClose={onClose}>
      <div className="text-center">
        <p
          className={cn(
            "italic text-base mb-5",
            "font-['Segoe_UI',Tahoma,Geneva,Verdana,sans-serif]"
          )}
        >
          {t("header.poweredBy")}&nbsp;{t("tenFramework")}.
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

export default AboutPopup;
