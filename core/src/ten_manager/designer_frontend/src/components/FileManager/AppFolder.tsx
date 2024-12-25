import * as React from "react";
import { FolderClosedIcon, FileIcon, FolderOpenIcon } from "lucide-react";

import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";

// TODO: remove mock data
const _MOCK_APP_FOLDER: IFileManagerData[] = [
  {
    name: "root",
    type: "folder",
    children: [
      {
        name: "folder1",
        type: "folder",
        children: [
          {
            name: "file1.txt",
            type: "file",
          },
          {
            name: "file2.txt",
            type: "file",
          },
          {
            name: "folder2",
            type: "folder",
            children: [
              {
                name: "file3.txt",
                type: "file",
              },
              {
                name: "file4.txt",
                type: "file",
              },
              {
                name: "folder3",
                type: "folder",
                children: [
                  {
                    name: "file5.txt",
                    type: "file",
                  },
                  {
                    name: "file6.txt",
                    type: "file",
                  },
                  {
                    name: "folder4",
                    type: "folder",
                    children: [
                      {
                        name: "file7.txt",
                        type: "file",
                      },
                      {
                        name: "file8.txt",
                        type: "file",
                      },
                      {
                        name: "folder5",
                        type: "folder",
                        children: [
                          {
                            name: "file9.txt",
                            type: "file",
                          },
                          {
                            name: "file10.txt",
                            type: "file",
                          },
                        ],
                      },
                    ],
                  },
                ],
              },
              {
                name: "file11.txt",
                type: "file",
              },
              {
                name: "file12.txt",
                type: "file",
              },
              {
                name: "folder6",
                type: "folder",
                children: [
                  {
                    name: "file13.txt",
                    type: "file",
                  },
                  {
                    name: "file14.txt",
                    type: "file",
                  },
                ],
              },
              {
                name: "folder7",
                type: "folder",
                children: [
                  {
                    name: "file15.txt",
                    type: "file",
                  },
                  {
                    name: "file16.txt",
                    type: "file",
                  },
                  {
                    name: "folder8",
                    type: "folder",
                    children: [
                      {
                        name: "file17.txt",
                        type: "file",
                      },
                      {
                        name: "file18.txt",
                        type: "file",
                      },
                    ],
                  },
                ],
              },
              {
                name: "file19.txt",
                type: "file",
              },
              {
                name: "file20.txt",
                type: "file",
              },
            ],
          },
          {
            name: "file21.txt",
            type: "file",
          },
          {
            name: "file22.txt",
            type: "file",
          },
          {
            name: "file23.txt",
            type: "file",
          },
          {
            name: "file24.txt",
            type: "file",
          },
          {
            name: "file25.txt",
            type: "file",
          },
          {
            name: "file26.txt",
            type: "file",
          },
          {
            name: "file27.txt",
            type: "file",
          },
          {
            name: "file28.txt",
            type: "file",
          },
          {
            name: "file29.txt",
            type: "file",
          },
          {
            name: "file30.txt",
            type: "file",
          },
          {
            name: "folder9",
            type: "folder",
            children: [
              {
                name: "file31.txt",
                type: "file",
              },
              {
                name: "file32.txt",
                type: "file",
              },
              {
                name: "folder10",
                type: "folder",
                children: [
                  {
                    name: "file33.txt",
                    type: "file",
                  },
                  {
                    name: "file34.txt",
                    type: "file",
                  },
                ],
              },
            ],
          },
          {
            name: "folder11",
            type: "folder",
            children: [
              {
                name: "file35.txt",
                type: "file",
              },
              {
                name: "file36.txt",
                type: "file",
              },
              {
                name: "folder12",
                type: "folder",
                children: [
                  {
                    name: "file37.txt",
                    type: "file",
                  },
                  {
                    name: "file38.txt",
                    type: "file",
                  },
                ],
              },
            ],
          },
          {
            name: "file39.txt",
            type: "file",
          },
          {
            name: "file40.txt",
            type: "file",
          },
          {
            name: "file41.txt",
            type: "file",
          },
          {
            name: "file42.txt",
            type: "file",
          },
          {
            name: "file43.txt",
            type: "file",
          },
          {
            name: "file44.txt",
            type: "file",
          },
          {
            name: "file45.txt",
            type: "file",
          },
          {
            name: "file46.txt",
            type: "file",
          },
          {
            name: "file47.txt",
            type: "file",
          },
          {
            name: "file48.txt",
            type: "file",
          },
          {
            name: "file49.txt",
            type: "file",
          },
          {
            name: "file50.txt",
            type: "file",
          },
        ],
      },
    ],
  },
];

export interface IFileManagerData {
  name: string;
  type: "file" | "folder";
  children?: IFileManagerData[];
}
export interface IFMTreeItem extends IFileManagerData {
  id: string;
  children?: IFMTreeItem[];
}

export function ThreeColumnFileManager(props: {
  // TODO: use enum
  allowSelectTypes?: ("file" | "folder")[];
  onSelect?: (path: string) => void;
  data?: IFileManagerData[];
  defaultSelectedId?: string;
  className?: string;
}) {
  const {
    onSelect,
    data = _MOCK_APP_FOLDER, // TODO: remove mock data
    defaultSelectedId = null,
    allowSelectTypes,
    className,
  } = props;
  const [selectedId, setSelectedId] = React.useState<string | null>(
    defaultSelectedId
  );

  const treeMemo = React.useMemo(() => parseFMTree(data), [data]);

  const colsMemo = React.useMemo(
    () => getFMTreeItemCols(treeMemo, selectedId),
    [treeMemo, selectedId]
  );

  const selectedPathStringMemo = React.useMemo(
    () => getFMTreeItemPath(treeMemo, selectedId),
    [treeMemo, selectedId]
  );

  const handleSelect = (id: string) => {
    setSelectedId(id);
    onSelect?.(getFMTreeItemPath(treeMemo, id));
  };

  const calculateDisabled = (data: IFMTreeItem) => {
    if (!allowSelectTypes || allowSelectTypes.length === 0) {
      return false;
    }
    return !allowSelectTypes.includes(data.type);
  };

  return (
    <div className={cn("w-full h-full space-y-2", className)}>
      <div
        className={cn(
          "w-full h-10 bg-gray-50 dark:bg-gray-900 rounded-lg",
          "px-4 flex items-center"
        )}
      >
        <span className="text-xs text-gray-500 dark:text-gray-400">
          {selectedPathStringMemo}
        </span>
      </div>
      <div
        className={cn(
          "flex py-2 w-full h-[calc(100%-3rem)]",
          "bg-gray-50 dark:bg-gray-900 rounded-lg"
        )}
      >
        <FileManagerColumn className="w-1/3 border-r border-gray-300">
          {colsMemo.length >= 3
            ? colsMemo[colsMemo.length - 3].map((item) => (
                <FileManagerColumnItem
                  key={item.id}
                  data={item}
                  selectStatus={
                    selectedId === item.id
                      ? "selected"
                      : selectedId?.startsWith(item.id)
                        ? "selected-parent"
                        : "unselected"
                  }
                  onClick={() => handleSelect(item.id)}
                  disabled={calculateDisabled(item)}
                />
              ))
            : colsMemo.length === 2
              ? colsMemo[colsMemo.length - 2].map((item) => (
                  <FileManagerColumnItem
                    key={item.id}
                    data={item}
                    selectStatus={
                      selectedId === item.id
                        ? "selected"
                        : selectedId?.startsWith(item.id)
                          ? "selected-parent"
                          : "unselected"
                    }
                    onClick={() => handleSelect(item.id)}
                    disabled={calculateDisabled(item)}
                  />
                ))
              : colsMemo[0].map((item) => (
                  <FileManagerColumnItem
                    key={item.id}
                    data={item}
                    selectStatus={
                      selectedId === item.id
                        ? "selected"
                        : selectedId?.startsWith(item.id)
                          ? "selected-parent"
                          : "unselected"
                    }
                    onClick={() => handleSelect(item.id)}
                    disabled={calculateDisabled(item)}
                  />
                ))}
        </FileManagerColumn>
        <FileManagerColumn className="w-1/3 border-r border-gray-300">
          {colsMemo.length >= 3
            ? colsMemo[colsMemo.length - 2].map((item) => (
                <FileManagerColumnItem
                  key={item.id}
                  data={item}
                  selectStatus={
                    selectedId === item.id
                      ? "selected"
                      : selectedId?.startsWith(item.id)
                        ? "selected-parent"
                        : "unselected"
                  }
                  onClick={() => handleSelect(item.id)}
                  disabled={calculateDisabled(item)}
                />
              ))
            : colsMemo.length === 2
              ? colsMemo[colsMemo.length - 1].map((item) => (
                  <FileManagerColumnItem
                    key={item.id}
                    data={item}
                    selectStatus={
                      selectedId === item.id
                        ? "selected"
                        : selectedId?.startsWith(item.id)
                          ? "selected-parent"
                          : "unselected"
                    }
                    onClick={() => handleSelect(item.id)}
                    disabled={calculateDisabled(item)}
                  />
                ))
              : []}
        </FileManagerColumn>
        <FileManagerColumn className="w-1/3 px-2 overflow-y-auto">
          {colsMemo.length >= 3
            ? colsMemo[colsMemo.length - 1].map((item) => (
                <FileManagerColumnItem
                  key={item.id}
                  data={item}
                  selectStatus={
                    selectedId === item.id
                      ? "selected"
                      : selectedId?.startsWith(item.id)
                        ? "selected-parent"
                        : "unselected"
                  }
                  onClick={() => handleSelect(item.id)}
                  disabled={calculateDisabled(item)}
                />
              ))
            : []}
        </FileManagerColumn>
      </div>
    </div>
  );
}

function FileManagerColumn(props: {
  className?: string;
  children?: React.ReactNode;
}) {
  const { className, children } = props;

  return <ul className={cn("px-2 overflow-y-auto", className)}>{children}</ul>;
}

function FileManagerColumnItem(props: {
  data: IFMTreeItem;
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

  return (
    <li key={data.name}>
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
        {data.type === "folder" &&
          (selectStatus === "selected" ||
            selectStatus === "selected-parent") && (
            <FolderOpenIcon className="" />
          )}
        {data.type === "folder" && selectStatus === "unselected" && (
          <FolderClosedIcon className="" />
        )}
        {data.type !== "folder" && <FileIcon className="" />}
        {data.name} {selectStatus === "selected" ? "(Selected)" : ""}
      </Button>
    </li>
  );
}

const parseFMTree = (data: IFileManagerData[]): IFMTreeItem[] => {
  const updateFMTree = (
    data: IFileManagerData[],
    currentDepth: number,
    parentId: string
  ): IFMTreeItem[] => {
    return data.map((item, index) => {
      const id = parentId ? `${parentId}-${index}` : `${index}`;
      const hasChildren = item.children && item.children.length > 0;
      const children = hasChildren
        ? updateFMTree(item.children!, currentDepth + 1, id)
        : [];

      return {
        ...item,
        id,
        children,
      };
    });
  };

  return updateFMTree(data, 0, "");
};

const getFMTreeItemCols = (
  tree: IFMTreeItem[],
  selectedId?: string | null
): Omit<IFMTreeItem, "children">[][] => {
  if (selectedId == null) {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    return [tree.map(({ children, ...rest }) => rest)];
  }

  const selectedIdParts = selectedId.split("-");
  const result: Omit<IFMTreeItem, "children">[][] = [];
  let currentLevelItems = tree;
  let currentId = "";

  for (let i = 0; i < selectedIdParts.length; i++) {
    const part = selectedIdParts[i];
    const selectedIndex = parseInt(part, 10);

    currentId = currentLevelItems[selectedIndex]?.id || "";

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    result.push(currentLevelItems.map(({ children, ...rest }) => rest));

    const selectedItem = currentLevelItems.find(
      (item) => item.id === currentId
    );
    if (selectedItem && selectedItem.children) {
      currentLevelItems = selectedItem.children;
    } else {
      break;
    }
  }

  if (currentLevelItems.length > 0) {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    result.push(currentLevelItems.map(({ children, ...rest }) => rest));
  }

  return result;
};

const getFMTreeItemPath = (
  tree: IFMTreeItem[],
  selectedId?: string | null
): string => {
  if (selectedId == null) {
    return tree[0].name;
  }

  const selectedIdParts = selectedId.split("-");
  let currentLevelItems = tree;
  let path = "";

  for (let i = 0; i < selectedIdParts.length; i++) {
    const part = selectedIdParts[i];
    const selectedIndex = parseInt(part, 10);

    const selectedItem = currentLevelItems[selectedIndex];
    if (selectedItem) {
      path += selectedItem.name + "/";
      if (selectedItem.children) {
        currentLevelItems = selectedItem.children;
      } else {
        break;
      }
    } else {
      break;
    }
  }

  return path.slice(0, -1); // Remove the trailing slash
};
