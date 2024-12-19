//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React from "react";

import Popup from "./Popup";
import { useTranslation } from "react-i18next";

interface AboutPopupProps {
  onClose: () => void;
}

const AboutPopup: React.FC<AboutPopupProps> = ({ onClose }) => {
  const { t } = useTranslation();

  return (
    <Popup title="About" onClose={onClose}>
      <div className="text-center">
        <p className="italic text-base font-['Segoe_UI',Tahoma,Geneva,Verdana,sans-serif] mb-5">
          Powered by TEN Framework.
        </p>
        <p className="my-1">
          {t("Official site")}:{" "}
          <a
            href="https://www.theten.ai/"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 underline"
          >
            https://www.theten.ai/
          </a>
        </p>
        <p className="my-1">
          Github:{" "}
          <a
            href="https://github.com/TEN-framework/"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 underline"
          >
            https://github.com/TEN-framework/
          </a>
        </p>
      </div>
    </Popup>
  );
};

export default AboutPopup;
