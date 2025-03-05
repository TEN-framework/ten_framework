//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { GithubIcon } from "lucide-react";

import { badgeVariants } from "@/components/ui/Badge";
import { cn } from "@/lib/utils";
import { useGHRepository } from "@/api/services/github";
import { formatNumberWithCommas } from "@/lib/utils";

export function GHStargazersCount(props: {
  owner: string;
  repo: string;
  className?: string;
}) {
  const { owner, repo, className } = props;

  const { repository, error, isLoading } = useGHRepository(owner, repo);

  if (isLoading || error) {
    return null;
  }

  return (
    <a
      href={`https://github.com/${owner}/${repo}`}
      target="_blank"
      rel="noopener noreferrer"
      className={cn(
        badgeVariants({ variant: "outline" }),
        "p-0 mx-1",
        className
      )}
    >
      <div
        className={cn(
          badgeVariants({ variant: "secondary" }),
          "h-[22px] rounded-r-none px-2"
        )}
      >
        <GithubIcon className="size-4" />
        {/* <span>Star</span> */}
      </div>
      <span
        className={cn(
          badgeVariants({ variant: "secondary" }),
          "bg-transparent rounded-l-none hover:bg-transparent"
        )}
      >
        {formatNumberWithCommas(repository?.stargazers_count || 0)}
      </span>
    </a>
  );
}
