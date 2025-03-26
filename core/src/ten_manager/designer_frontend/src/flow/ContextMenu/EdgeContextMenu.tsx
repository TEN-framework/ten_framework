//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React from "react";
import { useTranslation } from "react-i18next";
import { PencilIcon, ListCollapseIcon, TrashIcon } from "lucide-react";

import ContextMenu, { ContextMenuItem } from "@/flow/ContextMenu/ContextMenu";
import { CustomEdgeType } from "@/flow/CustomEdge";
import { dispatchCustomNodeActionPopup } from "@/utils/popup";

interface EdgeContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  edge: CustomEdgeType;
  onClose: () => void;
}

const EdgeContextMenu: React.FC<EdgeContextMenuProps> = ({
  visible,
  x,
  y,
  edge,
  onClose,
}) => {
  const { t } = useTranslation();

  const items: ContextMenuItem[] = [
    {
      label: t("action.viewDetails"),
      icon: <ListCollapseIcon />,
      onClick: () => {
        dispatchCustomNodeActionPopup("connections", edge.source, edge.target);
        onClose();
      },
    },
    {
      label: t("action.edit"),
      icon: <PencilIcon />,
      onClick: () => {
        onClose();
      },
    },
    {
      label: t("action.delete"),
      icon: <TrashIcon />,
      onClick: () => {
        onClose();
      },
    },
  ];

  return <ContextMenu visible={visible} x={x} y={y} items={items} />;
};

export default EdgeContextMenu;
