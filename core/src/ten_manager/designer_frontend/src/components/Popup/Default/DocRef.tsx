//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import {
  PopupBase,
  PopupInnerTabs,
  PopupInnerTab,
  PopupInnerTabContent,
} from "@/components/Popup/Base";
import { SpinnerLoading } from "@/components/Status/Loading";
import { DOC_REF_POPUP_ID } from "@/constants/widgets";
import { IDefaultWidget, EDefaultWidgetType } from "@/types/widgets";
import { useDocLink } from "@/api/services/doc";
import { EDocLinkKey } from "@/types/doc";
import { getCurrentWindowSize } from "@/utils/popup";
import { TEN_DOC_URL } from "@/constants";
import { useWidgetStore } from "@/store/widget";

export const DocRefPopup = (props: {
  tabs?: IDefaultWidget<{
    type: EDefaultWidgetType;
    base_dir?: string;
    scripts?: string[];
    doc_link_key?: string;
  }>[];
}) => {
  const { tabs } = props;

  const [activeTabId, setActiveTabId] = React.useState<string>(
    tabs?.[tabs.length - 1]?.sub_id || ""
  );

  const { t, i18n } = useTranslation();
  const { removeTabWidget, removeWidget } = useWidgetStore();
  const windowSize = getCurrentWindowSize();

  const handleTabIdUpdate = React.useCallback(
    (tabId?: string) => {
      setActiveTabId(tabId || tabs?.[tabs.length - 1]?.sub_id || "");
    },
    [tabs]
  );

  const handleCloseTab = React.useCallback(
    (id: string, subId: string) => {
      removeTabWidget(id, subId);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    []
  );

  const handleClosePopup = React.useCallback(() => {
    removeWidget(DOC_REF_POPUP_ID);
  }, [removeWidget]);

  if (!tabs) return null;

  return (
    <PopupBase
      id={DOC_REF_POPUP_ID}
      title={t("popup.doc.title")}
      contentClassName="p-0 flex flex-col h-full w-full"
      initialPosition="bottom-right"
      width={400}
      height={windowSize?.height ? windowSize?.height - 100 : 400}
      onTabIdUpdate={handleTabIdUpdate}
      onClose={handleClosePopup}
      resizable
    >
      <PopupInnerTabs>
        {tabs.map((tab, idx) => (
          <PopupInnerTab
            key={tab?.sub_id || idx}
            id={tab?.sub_id || ""}
            onClick={() => {
              setActiveTabId(tab.sub_id || "");
            }}
            onClose={() => handleCloseTab(tab.id, tab.sub_id || "")}
            isActive={activeTabId === tab.sub_id}
          >
            {tab.metadata?.doc_link_key}
          </PopupInnerTab>
        ))}
      </PopupInnerTabs>
      {tabs?.map((tab, idx) => (
        <PopupInnerTabContent
          key={tab?.sub_id || idx}
          isActive={activeTabId === tab?.sub_id}
        >
          <DocRefTabContent
            locale={i18n.language}
            queryKey={tab?.metadata?.doc_link_key as EDocLinkKey}
          />
        </PopupInnerTabContent>
      ))}
    </PopupBase>
  );
};

const DocRefTabContent = (props: { locale: string; queryKey: EDocLinkKey }) => {
  const { locale, queryKey } = props;

  const { data, error, isLoading } = useDocLink(queryKey, locale);

  const docPathMemo = React.useMemo(() => {
    const locale = data?.shortLocale;
    const text = data?.text;
    if (!locale || !text) {
      return undefined;
    }
    return `${locale}/${text}`;
  }, [data?.shortLocale, data?.text]);

  React.useEffect(() => {
    if (error) {
      toast.error(error.message);
    }
  }, [error]);

  if (isLoading) {
    return <SpinnerLoading className="h-full w-full" />;
  }

  if (!docPathMemo) return null;

  return (
    <iframe
      src={TEN_DOC_URL + docPathMemo}
      className="w-full h-full"
      title={queryKey}
      sandbox="allow-scripts allow-same-origin"
    />
  );
};
