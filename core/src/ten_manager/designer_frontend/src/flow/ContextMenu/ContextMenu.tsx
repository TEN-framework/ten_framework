//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React from "react";

export interface ContextMenuItem {
  label?: string;
  icon?: React.ReactNode;
  onClick?: () => void;
  separator?: boolean;
}

interface ContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
}

const ContextMenu: React.FC<ContextMenuProps> = ({ visible, x, y, items }) => {
  if (!visible) return null;

  return (
    <div
      className="fixed bg-white border border-gray-300 p-1.5 shadow-lg z-[9999] box-border"
      style={{
        left: x,
        top: y,
      }}
      onClick={(e) => e.stopPropagation()}
    >
      {items.map((item, index) => (
        <React.Fragment key={index}>
          {item.separator && <div className="h-px bg-gray-300 my-1.5"></div>}
          {!item.separator && (
            <div
              className="flex items-center px-2.5 py-1.5 whitespace-nowrap box-border cursor-pointer hover:bg-gray-100"
              onClick={item.onClick}
            >
              <span className="mr-2 flex items-center h-[1em] flex-shrink-0 w-5 justify-center">
                {item.icon || null}
              </span>
              <span className="flex-1 text-left">{item.label}</span>
            </div>
          )}
        </React.Fragment>
      ))}
    </div>
  );
};

export default ContextMenu;
