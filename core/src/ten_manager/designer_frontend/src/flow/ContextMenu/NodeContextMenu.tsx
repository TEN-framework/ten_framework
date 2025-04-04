//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React from "react";
import { useTranslation } from "react-i18next";
import {
  FilePenLineIcon,
  TerminalIcon,
  Trash2Icon,
  LogsIcon,
} from "lucide-react";

import ContextMenu, { ContextMenuItem } from "@/flow/ContextMenu/ContextMenu";

import type { TerminalData, EditorData } from "@/types/widgets";
import type { TCustomNode } from "@/types/flow";

interface NodeContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  node: TCustomNode;
  onClose: () => void;
  onLaunchTerminal: (data: TerminalData) => void;
  onLaunchEditor: (data: EditorData) => void;
  onLaunchLogViewer?: () => void;
}

const NodeContextMenu: React.FC<NodeContextMenuProps> = ({
  visible,
  x,
  y,
  node,
  onClose,
  onLaunchTerminal,
  onLaunchEditor,
  onLaunchLogViewer,
}) => {
  const { t } = useTranslation();

  const items: ContextMenuItem[] = [
    {
      label: t("action.edit") + " manifest.json",
      icon: <FilePenLineIcon />,
      onClick: () => {
        onClose();
        if (node?.data.url)
          onLaunchEditor({
            title: `${node.data.name} manifest.json`,
            content: "",
            url: `${node.data.url}/manifest.json`,
          });
      },
    },
    {
      label: t("action.edit") + " property.json",
      icon: <FilePenLineIcon />,
      onClick: () => {
        onClose();
        if (node?.data.url)
          onLaunchEditor({
            title: `${node.data.name} property.json`,
            content: "",
            url: `${node.data.url}/property.json`,
          });
      },
    },
    {
      separator: true,
    },
    {
      label: t("action.launchTerminal"),
      icon: <TerminalIcon />,
      onClick: () => {
        onClose();
        onLaunchTerminal({ title: node.data.name, url: node.data.url });
      },
    },
    {
      label: t("action.launchLogViewer"),
      icon: <LogsIcon />,
      onClick: () => {
        onClose();
        onLaunchLogViewer?.();
      },
    },
    {
      separator: true,
    },
    {
      label: t("action.delete"),
      icon: <Trash2Icon />,
      onClick: () => {
        onClose();
      },
    },
  ];

  return <ContextMenu visible={visible} x={x} y={y} items={items} />;
};

export default NodeContextMenu;
