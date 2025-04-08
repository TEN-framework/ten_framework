//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";

import { Popup } from "@/components/Popup/Popup";
import { useWidgetStore } from "@/store/widget";
import {
  GraphAddConnectionWidget,
  GraphAddNodeWidget,
} from "@/components/Widget/GraphsWidget";
import { EGraphActions } from "@/types/graphs";

import type { IGraphWidget } from "@/types/widgets";

const DEFAULT_WIDTH = 400;
const DEFAULT_HEIGHT = 300;

export const GraphPopup = (props: {
  id: string;
  metadata: IGraphWidget<{
    type: EGraphActions;
    base_dir: string;
    graph_name?: string;
  }>["metadata"];
}) => {
  const { id, metadata } = props;

  const { t } = useTranslation();
  const { removeWidget } = useWidgetStore();

  const handleClose = () => {
    removeWidget(id);
  };

  const titleMemo = React.useMemo(() => {
    switch (metadata.type) {
      case EGraphActions.ADD_NODE:
        return t("popup.graph.titleAddNode");
      case EGraphActions.ADD_CONNECTION:
        return t("popup.graph.titleAddConnection");
      default:
        return t("popup.graph.title");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [metadata.type]);

  return (
    <Popup
      id={id}
      title={titleMemo}
      onClose={handleClose}
      resizable={true}
      width={DEFAULT_WIDTH}
      height={DEFAULT_HEIGHT}
      contentClassName="px-0"
    >
      {metadata.type === EGraphActions.ADD_NODE && (
        <GraphAddNodeWidget
          {...metadata}
          postAddNodeActions={() => {
            removeWidget(id);
          }}
        />
      )}
      {metadata.type === EGraphActions.ADD_CONNECTION && (
        <GraphAddConnectionWidget {...metadata} />
      )}
    </Popup>
  );
};
