//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React, {
  useEffect,
  useState,
  useContext,
  useImperativeHandle,
} from "react";
import Editor from "@monaco-editor/react";
import { type editor } from "monaco-editor";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

import { retrieveFileContent, putFileContent } from "@/api/services/fileSystem";
import { ThemeProviderContext } from "@/components/theme-context";
import { useDialogStore } from "@/store/dialog";
import { useWidgetStore } from "@/store/widget";

import type { EditorData } from "@/types/widgets";

export interface EditorWidgetProps {
  id: string;
  data: EditorData;
}

export type TEditorOnClose = {
  postConfirm?: () => Promise<void>;
  postCancel?: () => Promise<void>;
  title?: string;
  content?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  hasUnsavedChanges?: boolean;
};

const EditorWidget = React.forwardRef<unknown, EditorWidgetProps>(
  ({ id, data }, ref) => {
    const [fileContent, setFileContent] = useState(data.content);

    const { t } = useTranslation();
    const { appendDialog, removeDialog } = useDialogStore();
    const { updateEditorStatus } = useWidgetStore();
    const editorRef = React.useRef<editor.IStandaloneCodeEditor | null>(null);

    // Fetch the specified file content from the backend.
    useEffect(() => {
      const fetchFileContent = async () => {
        try {
          const respData = await retrieveFileContent(data.url);
          setFileContent(respData.content);
        } catch (error) {
          console.error("Failed to fetch file content:", error);
          toast.error(t("toast.fetchFileContentFailed"));
        }
      };

      fetchFileContent();
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [data.url]);

    const saveFile = async (content: string) => {
      try {
        await putFileContent(data.url, { content });
        console.log(t("toast.saveFileSuccess"));
        toast.success(t("toast.saveFileSuccess"));
        // We can add UI prompts, such as displaying a success notification.
      } catch (error: unknown) {
        console.error("Failed to save file content:", error);
        toast.error(t("toast.saveFileFailed"), {
          description:
            error instanceof Error ? error.message : t("toast.saveFileFailed"),
        });
      }
    };

    const { theme } = useContext(ThemeProviderContext);

    useImperativeHandle(ref, () => ({
      id,
      onClose: ({
        postConfirm = () => Promise.resolve(),
        postCancel = () => Promise.resolve(),
        title = t("action.confirm"),
        content = t("popup.editor.confirmSaveFile"),
        confirmLabel = t("action.save"),
        cancelLabel = t("action.discard"),
        hasUnsavedChanges = false,
      }: TEditorOnClose) => {
        const dialogId = `confirm-dialog-imperative-${id}`;
        if (!hasUnsavedChanges) {
          removeDialog(dialogId);
          postConfirm();
          return;
        }
        appendDialog({
          id: dialogId,
          title,
          content,
          confirmLabel,
          cancelLabel,
          onConfirm: async () => {
            updateEditorStatus(id, false);
            await saveFile(fileContent);
            removeDialog(dialogId);
          },
          onCancel: async () => {
            removeDialog(dialogId);
          },
          postConfirm: postConfirm,
          postCancel: postCancel,
        });
      },
    }));

    return (
      <>
        <div className="p-0 box-border flex flex-col w-full h-full">
          <Editor
            theme={theme === "dark" ? "vs-dark" : "vs-light"}
            height="100%"
            defaultLanguage="json"
            value={fileContent}
            options={{
              readOnly: false,
              automaticLayout: true,
            }}
            onChange={(value) => {
              updateEditorStatus(id, true);
              setFileContent(value || "");
            }}
            onMount={(editor) => {
              editorRef.current = editor;
              // --- set context menu actions ---
              // reference: https://github.com/microsoft/monaco-editor/issues/1280#issuecomment-2420136963
              const keepIds = [
                "editor.action.clipboardCopyAction",
                "editor.action.clipboardCutAction",
                "editor.action.clipboardPasteAction",
                "editor.action.formatDocument",
                "vs.editor.ICodeEditor:1:save-file",
                "vs.actions.separator",
              ];
              const contextmenu = editor.getContribution(
                "editor.contrib.contextmenu"
              );
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              const realMethod = (contextmenu as any)._getMenuActions;
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              (contextmenu as any)._getMenuActions = function (...args: any[]) {
                const items = realMethod.apply(contextmenu, args);
                const filteredItems = items.filter(function (item: {
                  id: string;
                }) {
                  return keepIds.includes(item.id);
                });
                // Remove separator if it's the last item
                if (
                  filteredItems.length > 0 &&
                  filteredItems[filteredItems.length - 1].id ===
                    "vs.actions.separator"
                ) {
                  filteredItems.pop();
                }
                return filteredItems;
              };

              // --- set keyboard focus to the editor ---
              editor.focus();

              // --- add save-file action ---
              editor.addAction({
                id: "save-file",
                label: "Save",
                contextMenuGroupId: "navigation",
                contextMenuOrder: 1.5,
                run: async (ed) => {
                  // When the user clicks this menu item, first display a
                  // confirmation popup.
                  //   handleActionWithOptionalConfirm(
                  //     () => {
                  //       // Confirm before saving.
                  //       const currentContent = ed.getValue();
                  //       setFileContent(currentContent);
                  //       saveFile(currentContent);
                  //     },
                  //     true // Display a confirmation popup.
                  //   );
                  appendDialog({
                    id: `confirm-dialog-${id}`,
                    title: t("action.confirm"),
                    content: t("popup.editor.confirmSaveFile"),
                    onConfirm: async () => {
                      const currentContent = ed.getValue();
                      setFileContent(currentContent);
                      await saveFile(currentContent);
                      removeDialog(`confirm-dialog-${id}`);
                    },
                    onCancel: async () => {
                      removeDialog(`confirm-dialog-${id}`);
                    },
                  });
                },
              });
            }}
          />
        </div>
      </>
    );
  }
);

EditorWidget.displayName = "EditorWidget";

export default EditorWidget;
