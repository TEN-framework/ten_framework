//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { Button } from "@/components/ui/Button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/Dialog";
import { type IDialog, useDialogStore } from "@/store/dialog";
import { useTranslation } from "react-i18next";

export function GlobalDialogs() {
  const { dialogs } = useDialogStore();

  return (
    <>
      {dialogs.map((dialog) => (
        <GlobalDialog key={dialog.id} dialog={dialog} />
      ))}
    </>
  );
}

function GlobalDialog(props: { dialog: IDialog }) {
  const { dialog } = props;
  const { t } = useTranslation();

  return (
    <>
      <Dialog open={true}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{dialog.title}</DialogTitle>
            {dialog?.subTitle && (
              <DialogDescription>{dialog.subTitle}</DialogDescription>
            )}
          </DialogHeader>
          {dialog.content}
          <DialogFooter>
            {dialog.onCancel && (
              <Button
                variant="secondary"
                onClick={async () => {
                  await dialog.onCancel?.();
                  await dialog.postCancel?.();
                }}
              >
                {dialog.cancelLabel || t("action.cancel")}
              </Button>
            )}
            {dialog.onConfirm && (
              <Button
                variant="default"
                onClick={async () => {
                  await dialog.onConfirm?.();
                  await dialog.postConfirm?.();
                }}
              >
                {dialog.confirmLabel || t("action.confirm")}
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
