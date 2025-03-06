//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { StarIcon, BotIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/Button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/Tooltip";
import { GHIcon } from "@/components/Icons";
import { cn } from "@/lib/utils";
import { useGHRepository } from "@/api/services/github";
import { formatNumberWithCommas } from "@/lib/utils";
import { TEN_AGENT_URL } from "@/constants";

export function GHStargazersCount(props: {
  owner: string;
  repo: string;
  className?: string;
}) {
  const { owner, repo, className } = props;

  const { repository, error, isLoading } = useGHRepository(owner, repo);

  const shouldFallbackMemo = React.useMemo(() => {
    return isLoading || error || !repository?.stargazers_count;
  }, [isLoading, error, repository]);

  return (
    <Button asChild variant="ghost" size="sm">
      <a
        href={`https://github.com/${owner}/${repo}`}
        target="_blank"
        rel="noopener noreferrer"
        className={cn("flex items-center gap-1.5", "text-sm", className)}
      >
        {shouldFallbackMemo ? (
          <GHIcon className="size-4" />
        ) : (
          <>
            <StarIcon className="size-4 text-yellow-500" />
            <span>
              {formatNumberWithCommas(repository?.stargazers_count as number)}
            </span>
          </>
        )}
      </a>
    </Button>
  );
}

export function GHTryTENAgent(props: { className?: string }) {
  const { className } = props;

  const { t } = useTranslation();

  return (
    <TooltipProvider delayDuration={100}>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button asChild variant="ghost" size="sm">
            <a
              href={TEN_AGENT_URL}
              target="_blank"
              rel="noopener noreferrer"
              className={cn("flex items-center gap-1.5", "text-sm", className)}
            >
              <BotIcon className="size-4" />
              <span className="sr-only">{t("header.tryTENAgent")}</span>
            </a>
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{t("header.tryTENAgent")}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
