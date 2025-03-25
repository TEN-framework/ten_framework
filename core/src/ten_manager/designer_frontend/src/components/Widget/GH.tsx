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
import { SpinnerLoading } from "@/components/Status/Loading";
import { cn } from "@/lib/utils";
import { useGHRepository } from "@/api/services/github";
import { formatNumberWithCommas } from "@/lib/utils";
import { TEN_AGENT_URL } from "@/constants";
import { useHelpText } from "@/api/services/helpText";
import { EHelpTextKey } from "@/api/endpoints";

export function GHStargazersCount(props: {
  owner: string;
  repo: string;
  className?: string;
}) {
  const { owner, repo, className } = props;

  const { repository, error, isLoading } = useGHRepository(owner, repo);
  const {
    data: helpText,
    error: helpTextError,
    isLoading: helpTextIsLoading,
  } = useHelpText(EHelpTextKey.TEN_FRAMEWORK);

  const shouldFallbackMemo = React.useMemo(() => {
    return isLoading || error || !repository?.stargazers_count;
  }, [isLoading, error, repository]);

  React.useEffect(() => {
    if (helpTextError) {
      console.error(helpTextError);
    } else if (error) {
      console.error(error);
    }
  }, [helpTextError, error]);

  return (
    <TooltipProvider delayDuration={100}>
      <Tooltip>
        <TooltipTrigger asChild>
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
                    {formatNumberWithCommas(
                      repository?.stargazers_count as number
                    )}
                  </span>
                </>
              )}
            </a>
          </Button>
        </TooltipTrigger>
        <TooltipContent className="max-w-md">
          {helpTextIsLoading ? (
            <SpinnerLoading className="size-4" />
          ) : (
            <p>{helpText}</p>
          )}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export function GHTryTENAgent(props: { className?: string }) {
  const { className } = props;

  const { t } = useTranslation();
  const {
    data: helpText,
    error: helpTextError,
    isLoading: helpTextIsLoading,
  } = useHelpText(EHelpTextKey.TEN_AGENT);

  React.useEffect(() => {
    if (helpTextError) {
      console.error(helpTextError);
    }
  }, [helpTextError]);

  return (
    <TooltipProvider delayDuration={100}>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button asChild variant="ghost" size="icon">
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
        <TooltipContent className="max-w-md">
          {helpTextIsLoading ? (
            <SpinnerLoading className="size-4" />
          ) : (
            <p>{helpText}</p>
          )}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
