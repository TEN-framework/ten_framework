//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { EConnectionType } from "@/types/graphs";

export enum ECustomEventName {
  CustomNodeAction = "customNodeAction",
  BringToFrontPopup = "bringToFrontPopup",
}

export const dispatchCustomNodeActionPopup = (
  action: string,
  source: string,
  target?: string,
  metadata?: {
    filters?: {
      type?: EConnectionType;
      source?: boolean;
      target?: boolean;
    };
  }
) => {
  if (typeof window !== "undefined") {
    window.dispatchEvent(
      new CustomEvent(ECustomEventName.CustomNodeAction, {
        detail: { action, source, target, metadata },
      })
    );
  }
};

export const dispatchBringToFrontPopup = (widgetId: string) => {
  if (typeof window !== "undefined") {
    window.dispatchEvent(
      new CustomEvent(ECustomEventName.BringToFrontPopup, {
        detail: {
          id: widgetId,
        },
      })
    );
  }
};

export const getCurrentWindowSize = () => {
  if (typeof window !== "undefined") {
    return {
      width: window.innerWidth,
      height: window.innerHeight,
    };
  }
};
