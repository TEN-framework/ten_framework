//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { TEN_DEFAULT_BACKEND_WS_ENDPOINT } from "@/constants";

export const getWSEndpointFromWindow = (host?: string): string => {
  if (host) {
    return `ws://${host}`;
  } else if (typeof window !== "undefined" && window.location) {
    const { protocol, host } = window.location;
    const wsProtocol = protocol === "https:" ? "wss:" : "ws:";
    return `${wsProtocol}//${host}`;
  }
  return TEN_DEFAULT_BACKEND_WS_ENDPOINT;
};
