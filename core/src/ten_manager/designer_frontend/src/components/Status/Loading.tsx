import { LoaderCircleIcon } from "lucide-react";

import { cn } from "@/lib/utils";

export function SpinnerLoading(props: { className?: string }) {
  const { className } = props;

  return (
    <div className={cn("flex items-center justify-center", className)}>
      <LoaderCircleIcon className="animate-spin" />
    </div>
  );
}
