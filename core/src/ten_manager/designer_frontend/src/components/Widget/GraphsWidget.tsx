//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { CheckIcon, ChevronsUpDownIcon } from "lucide-react";

import { AddNodePayloadSchema } from "@/types/graphs";
import { Button } from "@/components/ui/Button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/Form";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { Input } from "@/components/ui/Input";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/Command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/Popover";
import { Textarea } from "@/components/ui/Textarea";
import { SpinnerLoading } from "@/components/Status/Loading";
import {
  useGraphs,
  postAddNode,
  retrieveGraphNodes,
  retrieveGraphConnections,
} from "@/api/services/graphs";
import { useAddons } from "@/api/services/addons";
import { cn } from "@/lib/utils";
import {
  generateRawNodes,
  generateRawEdges,
  updateNodesWithConnections,
  updateNodesWithAddonInfo,
  generateNodesAndEdges,
} from "@/flow/graph";
import { useAppStore, useFlowStore } from "@/store";

// eslint-disable-next-line react-refresh/only-export-components
export const resetNodesAndEdgesByGraphName = async (
  graphName: string,
  baseDir?: string | null
) => {
  const backendNodes = await retrieveGraphNodes(graphName, baseDir);
  const backendConnections = await retrieveGraphConnections(graphName, baseDir);
  const rawNodes = generateRawNodes(backendNodes);
  const [rawEdges, rawEdgeAddressMap] = generateRawEdges(backendConnections);
  const nodesWithConnections = updateNodesWithConnections(
    rawNodes,
    rawEdgeAddressMap
  );
  const nodesWithAddonInfo = await updateNodesWithAddonInfo(
    baseDir ?? "",
    nodesWithConnections
  );

  const { nodes: layoutedNodes, edges: layoutedEdges } = generateNodesAndEdges(
    nodesWithAddonInfo,
    rawEdges
  );

  return { nodes: layoutedNodes, edges: layoutedEdges };
};

export const GraphAddNodeWidget = (props: {
  base_dir: string;
  graph_name?: string;
  postAddNodeActions?: () => void | Promise<void>;
}) => {
  const { base_dir, graph_name, postAddNodeActions } = props;
  const [comboboxOpen, setComboboxOpen] = React.useState(false);
  const [isSubmitting, setIsSubmitting] = React.useState(false);

  const { t } = useTranslation();
  const { currentWorkspace } = useAppStore();
  const { setNodesAndEdges } = useFlowStore();

  const form = useForm<z.infer<typeof AddNodePayloadSchema>>({
    resolver: zodResolver(AddNodePayloadSchema),
    defaultValues: {
      graph_app_base_dir: base_dir,
      graph_name: graph_name,
      node_name: undefined,
      addon_name: undefined,
      addon_app_base_dir: undefined,
      extension_group_name: undefined,
      app_uri: undefined,
      property: "{}" as unknown as Record<string, unknown>,
    },
  });

  const onSubmit = async (data: z.infer<typeof AddNodePayloadSchema>) => {
    console.log("GraphAddNode", data);
    setIsSubmitting(true);
    try {
      await postAddNode(data);
      if (currentWorkspace?.graphName === data.graph_name) {
        const { nodes, edges } = await resetNodesAndEdgesByGraphName(
          data.graph_name,
          data.graph_app_base_dir
        );
        setNodesAndEdges(nodes, edges);
        postAddNodeActions?.();
      }
      toast.success(t("popup.node.addNodeSuccess"), {
        description: `Node ${data.node_name} added successfully`,
      });
    } catch (error) {
      console.error(error);
      toast.error(t("popup.node.addNodeError"), {
        description: error instanceof Error ? error.message : "Unknown error",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  const {
    graphs,
    isLoading: isGraphsLoading,
    error: graphError,
  } = useGraphs(base_dir);
  const {
    addons,
    isLoading: isAddonsLoading,
    error: addonError,
  } = useAddons(base_dir);

  React.useEffect(() => {
    if (graphError) {
      toast.error(t("popup.node.graphError"), {
        description: graphError.message,
      });
    }
    if (addonError) {
      toast.error(t("popup.node.addonError"), {
        description: addonError.message,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [graphError, addonError]);

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="space-y-4 w-full h-full overflow-y-auto px-2"
      >
        <FormField
          control={form.control}
          name="graph_app_base_dir"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.graphAppBaseDir")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.node.graphAppBaseDir")}
                  {...field}
                  readOnly
                  disabled
                />
              </FormControl>
              {/* <FormDescription>
                {t("popup.node.graphAppBaseDirDescription")}
              </FormDescription> */}
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="graph_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.graphName")}</FormLabel>
              <FormControl>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <SelectTrigger className="w-full" disabled={isGraphsLoading}>
                    <SelectValue placeholder={t("popup.node.graphName")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>{t("popup.node.graphName")}</SelectLabel>
                      {isGraphsLoading ? (
                        <SelectItem value={t("popup.node.graphName")}>
                          <SpinnerLoading className="size-4" />
                        </SelectItem>
                      ) : (
                        graphs?.map((graph) => (
                          <SelectItem key={graph.name} value={graph.name}>
                            {graph.name}
                          </SelectItem>
                        ))
                      )}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="node_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.nodeName")}</FormLabel>
              <FormControl>
                <Input placeholder={t("popup.node.nodeName")} {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="addon_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.addonName")}</FormLabel>
              <FormControl>
                <Popover open={comboboxOpen} onOpenChange={setComboboxOpen}>
                  <PopoverTrigger asChild>
                    <Button
                      disabled={isAddonsLoading}
                      variant="outline"
                      role="combobox"
                      aria-expanded={comboboxOpen}
                      className="w-full justify-between"
                    >
                      {isAddonsLoading && <SpinnerLoading className="size-4" />}
                      {field.value
                        ? addons?.find((addon) => addon.name === field.value)
                            ?.name
                        : t("popup.node.addonName")}
                      {/* eslint-disable-next-line max-len */}
                      <ChevronsUpDownIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-full p-0">
                    <Command>
                      <CommandInput placeholder={t("popup.node.searchAddon")} />
                      <CommandList>
                        <CommandEmpty>
                          {t("popup.node.noAddonFound")}
                        </CommandEmpty>
                        <CommandGroup>
                          {addons?.map((addon) => (
                            <CommandItem
                              key={addon.name}
                              value={addon.name}
                              onSelect={(currentValue) => {
                                field.onChange(currentValue);
                                setComboboxOpen(false);
                              }}
                            >
                              <CheckIcon
                                className={cn(
                                  "mr-2 h-4 w-4",
                                  field.value === addon.name
                                    ? "opacity-100"
                                    : "opacity-0"
                                )}
                              />
                              {addon.name}
                            </CommandItem>
                          ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        {/* <FormField
          control={form.control}
          name="addon_app_base_dir"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.addonAppBaseDir")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.node.addonAppBaseDir")}
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        /> */}

        {/* <FormField
          control={form.control}
          name="extension_group_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.extensionGroupName")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.node.extensionGroupName")}
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        /> */}

        {/* <FormField
          control={form.control}
          name="app_uri"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.appUri")}</FormLabel>
              <FormControl>
                <Input placeholder={t("popup.node.appUri")} {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        /> */}

        <FormField
          control={form.control}
          name="property"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.node.property")}</FormLabel>
              <FormControl>
                <Textarea
                  placeholder={t("popup.node.property")}
                  defaultValue={JSON.stringify({})}
                  value={field.value as unknown as string}
                  onChange={(e) => {
                    field.onChange(e.target.value);
                  }}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <Button
          type="submit"
          disabled={isAddonsLoading || isGraphsLoading || isSubmitting}
        >
          {isSubmitting ? (
            <SpinnerLoading className="size-4" />
          ) : (
            t("popup.node.addNode")
          )}
        </Button>
      </form>
    </Form>
  );
};
