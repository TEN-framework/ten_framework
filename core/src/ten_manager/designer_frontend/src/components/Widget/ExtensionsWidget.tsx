import * as React from "react";
import { useTranslation } from "react-i18next";
import { FilterIcon } from "lucide-react";
import { DropdownMenuCheckboxItemProps } from "@radix-ui/react-dropdown-menu";

import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";

type Checked = DropdownMenuCheckboxItemProps["checked"];

export function ExtensionsWidget() {
  const [search, setSearch] = React.useState("");
  const deferredSearch = React.useDeferredValue(search);

  const [showInstalled, setShowInstalled] = React.useState<Checked>(true);
  const [showUninstalled, setShowUninstalled] = React.useState<Checked>(true);

  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-2 w-full">
      <div className="flex w-full items-center justify-between space-x-2">
        <Input
          type="text"
          placeholder={t("action.searchPlaceholder")}
          className="w-full"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <div className="flex items-center space-x-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="icon">
                <FilterIcon />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-56">
              <DropdownMenuLabel>{t("action.filter")}</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuCheckboxItem
                checked={showInstalled}
                onCheckedChange={setShowInstalled}
              >
                {t("extensions.installed")}
              </DropdownMenuCheckboxItem>
              <DropdownMenuCheckboxItem
                checked={showUninstalled}
                onCheckedChange={setShowUninstalled}
              >
                {t("extensions.uninstalled")}
              </DropdownMenuCheckboxItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}
