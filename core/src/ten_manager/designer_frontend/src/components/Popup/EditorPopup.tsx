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
import EditorWidget from "@/components/Widget/EditorWidget";
import { EWidgetDisplayType } from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";

import type { EditorData } from "@/types/widgets";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

interface EditorPopupProps {
  id: string;
  data: EditorData;
  onClose: () => void;
}

type TEditorPopupRef = {
  postConfirm?: () => Promise<void>;
  postCancel?: () => Promise<void>;
  title?: string;
  content?: string;
};

type TEditorPopupRefActions = {
  onClose: ({
    postConfirm,
    postCancel,
    title,
    content,
  }: TEditorPopupRef) => void;
};

const EditorPopup: React.FC<EditorPopupProps> = ({ id, data, onClose }) => {
  const editorRef = React.useRef<TEditorPopupRef>(null);
  const { t } = useTranslation();
  const { updateWidgetDisplayType } = useWidgetStore();

  const handleSave = () => {
    (editorRef.current as TEditorPopupRefActions)?.onClose({
      postConfirm: async () => {},
      postCancel: async () => {},
    });
  };

  const handlePinToDock = () => {
    (editorRef.current as TEditorPopupRefActions)?.onClose({
      title: t("action.confirm"),
      content: t("popup.editor.confirmSaveChanges"),
      postConfirm: async () => {
        updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
      },
      postCancel: async () => {
        updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
      },
    });
  };

  const handleClose = () => {
    (editorRef.current as TEditorPopupRefActions)?.onClose({
      title: t("action.confirm"),
      content: t("popup.editor.confirmSaveChanges"),
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
        title={data.title}
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
