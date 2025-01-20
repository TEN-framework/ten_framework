//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { SaveIcon, PinIcon } from "lucide-react";

import Popup from "@/components/Popup/Popup";
import EditorWidget, {
  type TEditorOnClose,
} from "@/components/Widget/EditorWidget";
import { EWidgetDisplayType } from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";

import type { EditorData } from "@/types/widgets";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

interface EditorPopupProps {
  id: string;
  data: EditorData;
  onClose: () => void;
  hasUnsavedChanges?: boolean;
}

type TEditorPopupRefActions = {
  onClose: ({
    postConfirm,
    postCancel,
    title,
    content,
    hasUnsavedChanges,
  }: TEditorOnClose) => void;
};

const EditorPopup: React.FC<EditorPopupProps> = ({
  id,
  data,
  onClose,
  hasUnsavedChanges = false,
}) => {
  const editorRef = React.useRef<TEditorPopupRefActions>(null);
  const { t } = useTranslation();
  const { updateWidgetDisplayType } = useWidgetStore();

  const handleSave = () => {
    editorRef.current?.onClose({
      hasUnsavedChanges,
      postConfirm: async () => {},
      postCancel: async () => {},
    });
  };

  const handlePinToDock = () => {
    editorRef.current?.onClose({
      title: t("action.confirm"),
      content: t("popup.editor.confirmSaveChanges"),
      hasUnsavedChanges,
      postConfirm: async () => {
        updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
      },
      postCancel: async () => {
        updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
      },
    });
  };

  const handleClose = () => {
    editorRef.current?.onClose({
      title: t("action.confirm"),
      content: t("popup.editor.confirmSaveChanges"),
      hasUnsavedChanges,
      postConfirm: async () => {
        await onClose();
      },
      postCancel: async () => {
        await onClose();
      },
    });
  };

  return (
    <>
      <Popup
        id={id}
        title={
          <div className="flex items-center gap-1.5">
            <span className="font-medium text-foreground/90 font-sans">
              {data.title}
            </span>
            <span className="text-foreground/50 text-sm font-sans">
              {hasUnsavedChanges ? "(unsaved)" : ""}
            </span>
          </div>
        }
        onClose={handleClose}
        preventFocusSteal={true}
        resizable={true}
        initialWidth={DEFAULT_WIDTH}
        initialHeight={DEFAULT_HEIGHT}
        contentClassName="p-0"
        customActions={[
          {
            id: "save-file",
            label: t("action.save"),
            Icon: SaveIcon,
            onClick: handleSave,
          },
          {
            id: "pin-to-dock",
            label: t("action.pinToDock"),
            Icon: PinIcon,
            onClick: handlePinToDock,
          },
        ]}
      >
        <EditorWidget id={id} data={data} ref={editorRef} />
      </Popup>
    </>
  );
};

export default EditorPopup;
