//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";

import { PopupBase } from "@/components/Popup/Base";
import { useWidgetStore } from "@/store/widget";
import {
  GraphAddConnectionWidget,
  GraphAddNodeWidget,
  GraphUpdateNodePropertyWidget,
} from "@/components/Widget/GraphsWidget";
import { EGraphActions } from "@/types/graphs";

import type { IGraphWidget } from "@/types/widgets";
import type { TCustomNode } from "@/types/flow";

const DEFAULT_WIDTH = 400;
const DEFAULT_HEIGHT = 300;

export const GraphPopup = (props: {
  id: string;
  metadata: IGraphWidget<{
    type: EGraphActions;
    base_dir: string;
    graph_name?: string;
    node?: TCustomNode;
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
      case EGraphActions.UPDATE_NODE_PROPERTY:
        return t("popup.graph.titleUpdateNodePropertyByName", {
          name: metadata.node?.data.name,
        });
      default:
        return t("popup.graph.title");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [metadata.type, metadata.node]);

  return (
    <PopupBase
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
        <GraphAddConnectionWidget
          {...metadata}
          postAddConnectionActions={() => {
            removeWidget(id);
          }}
        />
      )}
      {metadata.type === EGraphActions.UPDATE_NODE_PROPERTY &&
        metadata.node && (
          <GraphUpdateNodePropertyWidget
            {...metadata}
            node={metadata.node}
            postUpdateNodePropertyActions={() => {
              removeWidget(id);
            }}
          />
        )}
    </PopupBase>
  );
};
