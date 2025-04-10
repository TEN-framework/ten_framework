//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import { PopupBase } from "@/components/Popup/Base";
import { SpinnerLoading } from "@/components/Status/Loading";
import { DOC_REF_POPUP_ID } from "@/constants/widgets";
import { IDefaultWidget } from "@/types/widgets";
import { useDocLink } from "@/api/services/doc";
import { EDocLinkKey } from "@/types/doc";
import { getCurrentWindowSize } from "@/utils/popup";
import { TEN_DOC_URL } from "@/constants";
import { useWidgetStore } from "@/store/widget";

export const DocRefPopupTitle = (props: { name: string }) => {
  const { name } = props;
  const { t } = useTranslation();

  return t("popup.doc.title", { name });
};

export const DocRefPopupContent = (props: { widget: IDefaultWidget }) => {
  const { widget } = props;
  const { doc_link_key } = widget.metadata;

  const { i18n } = useTranslation();

  if (!doc_link_key) return null;

  return <DocRefTabContent locale={i18n.language} queryKey={doc_link_key} />;
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
