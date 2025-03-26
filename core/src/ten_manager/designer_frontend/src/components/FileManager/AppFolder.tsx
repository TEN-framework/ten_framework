//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { FolderClosedIcon, FileIcon, FolderOpenIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { SpinnerLoading } from "@/components/Status/Loading";
import { cn } from "@/lib/utils";
import type { IFMItem } from "@/components/FileManager/utils";
import { EFMItemType, calculateDirDepth } from "@/components/FileManager/utils";

const DEFAULT_SELECTED_PATH = "/";

enum EFMItemSelectedStatus {
  UNSELECTED = "unselected",
  SELECTED = "selected",
  SELECTED_PARENT = "selected-parent",
}

function FileManagerColumn(props: {
  className?: string;
  children?: React.ReactNode;
  isLoading?: boolean;
  style?: React.CSSProperties;
}) {
  const { className, children, isLoading, style } = props;

  return (
    <ul
      className={cn(
        "px-2 overflow-y-auto",
        {
          "flex items-center justify-center text-gray-500 dark:text-gray-400":
            isLoading,
        },
        className
      )}
      style={style}
    >
      {isLoading ? <SpinnerLoading /> : children}
    </ul>
  );
}

function FileManagerColumnItem(props: {
  data: IFMItem;
  onClick?: () => void;
  className?: string;
  selectStatus?: "unselected" | "selected" | "selected-parent";
  disabled?: boolean;
}) {
  const {
    data,
    onClick,
    className,
    selectStatus = "unselected",
    disabled,
  } = props;

  const { t } = useTranslation();

  return (
    <li key={data.path}>
      <Button
        variant="ghost"
        size="sm"
        className={cn(
          "w-full justify-start",
          "hover:bg-gray-200 dark:hover:bg-gray-900",
          {
            "bg-gray-300 dark:bg-gray-800": selectStatus === "selected",
            "bg-gray-200 dark:bg-gray-900": selectStatus === "selected-parent",
          },
          className
        )}
        onClick={onClick}
        disabled={disabled}
      >
        {data.type === EFMItemType.FOLDER &&
          (selectStatus === "selected" ||
            selectStatus === "selected-parent") && (
            <FolderOpenIcon className="" />
          )}
        {data.type === EFMItemType.FOLDER && selectStatus === "unselected" && (
          <FolderClosedIcon className="" />
        )}
        {data.type !== EFMItemType.FOLDER && <FileIcon className="" />}
        <span className="overflow-hidden text-ellipsis whitespace-nowrap">
          {data.name}{" "}
          {selectStatus === "selected" ? `(${t("action.selected")})` : ""}
        </span>
      </Button>
    </li>
  );
}

export function FileManager(props: {
  allowSelectTypes?: EFMItemType[];
  onSelect?: (path: string) => void;
  data?: IFMItem[][];
  selectedPath?: string;
  className?: string;
  isLoading?: boolean;
  colWidth?: number;
}) {
  const {
    onSelect,
    data,
    selectedPath = DEFAULT_SELECTED_PATH,
    allowSelectTypes,
    className,
    isLoading,
    colWidth,
  } = props;

  const colsEndEleRef = React.useRef<HTMLDivElement>(null);

  const colsMemo = React.useMemo(() => {
    const selectedPathDepth = calculateDirDepth(selectedPath);
    const currentPathDepth = Math.max(1, selectedPathDepth + 1);
    const levelData = data?.slice(1, currentPathDepth + 1) || [];
    const filteredData = levelData
      .reverse()
      .reduce(
        (
          prev: (IFMItem & {
            selectedStatus?: EFMItemSelectedStatus;
          })[][],
          cur: IFMItem[],
          curIdx: number
        ): (IFMItem & {
          selectedStatus?: EFMItemSelectedStatus;
        })[][] => {
          if (curIdx === 0) {
            const filteredItems = cur
              .filter((item) => item.path.startsWith(selectedPath))
              .map((item) => ({
                ...item,
                selectedStatus: EFMItemSelectedStatus.UNSELECTED,
              }));
            prev.push(filteredItems);
            return prev;
          }
          const selectedPathParentList = selectedPath
            .split("/")
            .slice(0, -curIdx)
            .join("/");
          const selectedPathCurrentList = selectedPath
            .split("/")
            .slice(0, -curIdx + 1)
            .join("/");
          const filteredItems = cur
            .filter((item) => item.path.startsWith(selectedPathParentList))
            .map((item) => ({
              ...item,
              selectedStatus: EFMItemSelectedStatus.UNSELECTED,
              selectedPathParentList,
              selectedPathCurrentList,
            }));
          if (curIdx === 1) {
            const selectedItem = filteredItems.find(
              (item) => item.path === selectedPath
            );
            if (selectedItem) {
              selectedItem.selectedStatus = EFMItemSelectedStatus.SELECTED;
            }
          } else {
            const selectedParentItem = filteredItems.find(
              (item) => item.path === selectedPathCurrentList
            );
            if (selectedParentItem) {
              selectedParentItem.selectedStatus =
                EFMItemSelectedStatus.SELECTED_PARENT;
            }
          }
          prev.push(filteredItems);
          return prev;
        },
        [] as (IFMItem & {
          selectedStatus?: EFMItemSelectedStatus;
        })[][]
      )
      .reverse();
    return filteredData;
  }, [data, selectedPath]);

  React.useEffect(() => {
    if (colsEndEleRef.current) {
      colsEndEleRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [colsMemo?.length]);

  return (
    <div className={cn("w-full h-full space-y-2", className)}>
      <Input className="h-10" value={selectedPath} readOnly />
      <div
        className={cn(
          "flex py-2 w-full h-[calc(100%-3rem)]",
          "bg-gray-50 dark:bg-gray-900 rounded-lg",
          "overflow-x-auto"
        )}
      >
        <div className="flex w-fit">
          {colsMemo?.map((item, idx) => (
            <FileManagerColumn
              key={idx}
              className="border-r border-gray-300"
              style={{ width: colWidth }}
              isLoading={isLoading && idx === colsMemo.length - 1}
            >
              {item.map((item) => (
                <FileManagerColumnItem
                  key={item.path}
                  data={item}
                  onClick={() => onSelect?.(item.path)}
                  selectStatus={item.selectedStatus}
                  disabled={
                    allowSelectTypes
                      ? !allowSelectTypes.includes(item.type)
                      : false
                  }
                />
              ))}
            </FileManagerColumn>
          ))}
        </div>
        <div ref={colsEndEleRef} />
      </div>
    </div>
  );
}
