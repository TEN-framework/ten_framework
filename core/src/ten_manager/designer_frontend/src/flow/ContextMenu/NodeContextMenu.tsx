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
import { toast } from "sonner";

import ContextMenu, {
  EContextMenuItemType,
  type IContextMenuItem,
} from "@/flow/ContextMenu/ContextMenu";
import { useDialogStore, useFlowStore } from "@/store";
import { postDeleteNode } from "@/api/services/graphs";
// eslint-disable-next-line max-len
import { resetNodesAndEdgesByGraphName } from "@/components/Widget/GraphsWidget";

import type { TerminalData, EditorData } from "@/types/widgets";
import type { TCustomNode } from "@/types/flow";

interface NodeContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  node: TCustomNode;
  baseDir?: string | null;
  graphName?: string | null;
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
  baseDir,
  graphName,
  onClose,
  onLaunchTerminal,
  onLaunchEditor,
  onLaunchLogViewer,
}) => {
  const { t } = useTranslation();
  const { appendDialog, removeDialog } = useDialogStore();
  const { setNodesAndEdges } = useFlowStore();

  const items: IContextMenuItem[] = [
    {
      _type: EContextMenuItemType.SUB_MENU_BUTTON,
      label: t("action.edit") + " " + t("extensionStore.extension"),
      icon: <FilePenLineIcon />,
      items: [
        {
          _type: EContextMenuItemType.BUTTON,
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
          _type: EContextMenuItemType.BUTTON,
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
      ],
    },
    {
      _type: EContextMenuItemType.SEPARATOR,
    },
    {
      _type: EContextMenuItemType.BUTTON,
      label: t("action.update") + " " + t("popup.node.properties"),
      icon: <FilePenLineIcon />,
      onClick: () => {
        onClose();
      },
    },
    {
      _type: EContextMenuItemType.SEPARATOR,
    },
    {
      _type: EContextMenuItemType.BUTTON,
      label: t("action.launchTerminal"),
      icon: <TerminalIcon />,
      onClick: () => {
        onClose();
        onLaunchTerminal({ title: node.data.name, url: node.data.url });
      },
    },
    {
      _type: EContextMenuItemType.BUTTON,
      label: t("action.launchLogViewer"),
      icon: <LogsIcon />,
      onClick: () => {
        onClose();
        onLaunchLogViewer?.();
      },
    },
    {
      _type: EContextMenuItemType.SEPARATOR,
    },
    {
      _type: EContextMenuItemType.BUTTON,
      label: t("action.delete"),
      disabled: !baseDir || !graphName,
      icon: <Trash2Icon />,
      onClick: () => {
        onClose();
        appendDialog({
          id: "delete-node-dialog-" + node.data.name,
          title: t("action.delete"),
          content: t("action.deleteNodeConfirmationWithName", {
            name: node.data.name,
          }),
          variant: "destructive",
          onCancel: async () => {
            removeDialog("delete-node-dialog-" + node.data.name);
          },
          onConfirm: async () => {
            if (!baseDir || !graphName) {
              removeDialog("delete-node-dialog-" + node.data.name);
              return;
            }
            try {
              await postDeleteNode({
                base_dir: baseDir,
                graph_name: graphName,
                node_name: node.data.name,
                addon_name: node.data.addon,
                extension_group_name: node.data.extension_group,
              });
              toast.success(t("popup.node.deleteNodeSuccess"), {
                description: `${node.data.name}`,
              });
              const { nodes, edges } = await resetNodesAndEdgesByGraphName(
                graphName,
                baseDir
              );
              setNodesAndEdges(nodes, edges);
            } catch (error: unknown) {
              toast.error(t("action.deleteNodeFailed"), {
                description:
                  error instanceof Error ? error.message : "Unknown error",
              });
              console.error(error);
            } finally {
              removeDialog("delete-node-dialog-" + node.data.name);
            }
          },
        });
      },
    },
  ];

  return <ContextMenu visible={visible} x={x} y={y} items={items} />;
};

export default NodeContextMenu;
