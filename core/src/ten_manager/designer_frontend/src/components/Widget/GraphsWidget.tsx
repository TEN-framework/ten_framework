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

import {
  AddNodePayloadSchema,
  AddConnectionPayloadSchema,
  EConnectionType,
} from "@/types/graphs";
import { Button } from "@/components/ui/Button";
import {
  Form,
  FormControl,
  //   FormDescription,
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
  postAddConnection,
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
      toast.success(t("popup.graph.addNodeSuccess"), {
        description: `Node ${data.node_name} added successfully`,
      });
    } catch (error) {
      console.error(error);
      toast.error(t("popup.graph.addNodeError"), {
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
      toast.error(t("popup.graph.graphError"), {
        description: graphError.message,
      });
    }
    if (addonError) {
      toast.error(t("popup.graph.addonError"), {
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
              <FormLabel>{t("popup.graph.graphAppBaseDir")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.graph.graphAppBaseDir")}
                  {...field}
                  readOnly
                  disabled
                />
              </FormControl>
              {/* <FormDescription>
                {t("popup.graph.graphAppBaseDirDescription")}
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
              <FormLabel>{t("popup.graph.graphName")}</FormLabel>
              <FormControl>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <SelectTrigger className="w-full" disabled={isGraphsLoading}>
                    <SelectValue placeholder={t("popup.graph.graphName")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>{t("popup.graph.graphName")}</SelectLabel>
                      {isGraphsLoading ? (
                        <SelectItem value={t("popup.graph.graphName")}>
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
              <FormLabel>{t("popup.graph.nodeName")}</FormLabel>
              <FormControl>
                <Input placeholder={t("popup.graph.nodeName")} {...field} />
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
              <FormLabel>{t("popup.graph.addonName")}</FormLabel>
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
                        : t("popup.graph.addonName")}
                      {/* eslint-disable-next-line max-len */}
                      <ChevronsUpDownIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-full p-0">
                    <Command>
                      <CommandInput
                        placeholder={t("popup.graph.searchAddon")}
                      />
                      <CommandList>
                        <CommandEmpty>
                          {t("popup.graph.noAddonFound")}
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
              <FormLabel>{t("popup.graph.addonAppBaseDir")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.graph.addonAppBaseDir")}
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
              <FormLabel>{t("popup.graph.extensionGroupName")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.graph.extensionGroupName")}
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
              <FormLabel>{t("popup.graph.appUri")}</FormLabel>
              <FormControl>
                <Input placeholder={t("popup.graph.appUri")} {...field} />
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
              <FormLabel>{t("popup.graph.property")}</FormLabel>
              <FormControl>
                <Textarea
                  placeholder={t("popup.graph.property")}
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
            t("popup.graph.addNode")
          )}
        </Button>
      </form>
    </Form>
  );
};

const AddConnectionFormSchema = AddConnectionPayloadSchema.extend({
  //   dest_app_dir: z.string(),
  //   dest_graph_name: z.string(),
});

export const GraphAddConnectionWidget = (props: {
  base_dir: string;
  app_uri?: string | null;
  graph_name?: string;
  postAddConnectionActions?: () => void | Promise<void>;
}) => {
  const { base_dir, app_uri, graph_name, postAddConnectionActions } = props;
  const [isSubmitting, setIsSubmitting] = React.useState(false);

  const { t } = useTranslation();
  const { nodes, setNodesAndEdges } = useFlowStore();
  const {
    graphs,
    isLoading: isGraphsLoading,
    error: graphError,
  } = useGraphs(base_dir);
  const { currentWorkspace } = useAppStore();

  const form = useForm<z.infer<typeof AddConnectionFormSchema>>({
    resolver: zodResolver(AddConnectionFormSchema),
    defaultValues: {
      base_dir: base_dir,
      graph_name: graph_name,
      src_app: app_uri,
      src_extension: undefined,
      msg_type: EConnectionType.CMD,
      msg_name: undefined,
      dest_app: app_uri,
      //   dest_app_dir: base_dir, // extended field, cause dest_app is nullable
      //   dest_graph_name: graph_name, // extended field, target graph > nodes
      dest_extension: undefined,
    },
  });

  const onSubmit = async (data: z.infer<typeof AddConnectionFormSchema>) => {
    setIsSubmitting(true);
    try {
      const payload = AddConnectionPayloadSchema.parse(data);
      if (payload.src_extension === payload.dest_extension) {
        throw new Error(t("popup.graph.sameNodeError"));
        return;
      }
      console.log("GraphAddConnection", payload);
      await postAddConnection(payload);
      if (
        currentWorkspace?.graphName === data.graph_name &&
        data.base_dir === base_dir
      ) {
        const { nodes, edges } = await resetNodesAndEdgesByGraphName(
          data.graph_name,
          data.base_dir
        );
        setNodesAndEdges(nodes, edges);
        postAddConnectionActions?.();
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Unknown error");
      console.error(error);
    } finally {
      setIsSubmitting(false);
    }
  };

  React.useEffect(() => {
    if (graphError) {
      toast.error(t("popup.graph.graphError"), {
        description: graphError.message,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [graphError]);

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="space-y-4 w-full h-full overflow-y-auto px-2"
      >
        <FormField
          control={form.control}
          name="base_dir"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.graphAppBaseDir")}</FormLabel>
              <FormControl>
                <Input
                  placeholder={t("popup.graph.graphAppBaseDir")}
                  {...field}
                  readOnly
                  disabled
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="graph_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.graphName")}</FormLabel>
              <FormControl>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <SelectTrigger className="w-full" disabled={isGraphsLoading}>
                    <SelectValue placeholder={t("popup.graph.graphName")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>{t("popup.graph.graphName")}</SelectLabel>
                      {isGraphsLoading ? (
                        <SelectItem value={t("popup.graph.graphName")}>
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
          name="src_extension"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.srcExtension")}</FormLabel>
              <FormControl>
                <Select onValueChange={field.onChange} value={field.value}>
                  <SelectTrigger className="w-full overflow-hidden">
                    <SelectValue placeholder={t("popup.graph.srcExtension")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>{t("popup.graph.srcExtension")}</SelectLabel>
                      {nodes.map((node) => (
                        <SelectItem key={node.id} value={node.data.name}>
                          {node.data.name}
                        </SelectItem>
                      ))}
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
          name="msg_type"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.messageType")}</FormLabel>
              <FormControl>
                <Select onValueChange={field.onChange} value={field.value}>
                  <SelectTrigger className="w-full overflow-hidden">
                    <SelectValue placeholder={t("popup.graph.messageType")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>{t("popup.graph.messageType")}</SelectLabel>
                      {Object.values(EConnectionType).map((type) => (
                        <SelectItem key={type} value={type}>
                          {type}
                        </SelectItem>
                      ))}
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
          name="msg_name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.messageName")}</FormLabel>
              <FormControl>
                <Input placeholder={t("popup.graph.messageName")} {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="dest_extension"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.destExtension")}</FormLabel>
              <FormControl>
                <Select
                  onValueChange={field.onChange}
                  value={field.value ?? undefined}
                >
                  <SelectTrigger className="w-full overflow-hidden">
                    <SelectValue placeholder={t("popup.graph.destExtension")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>
                        {t("popup.graph.destExtension")}
                      </SelectLabel>
                      {nodes
                        .filter((i) => i.id !== form.watch("src_extension"))
                        .map((node) => (
                          <SelectItem key={node.id} value={node.data.name}>
                            {node.data.name}
                          </SelectItem>
                        ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting && <SpinnerLoading className="size-4" />}
          {t("popup.graph.addConnection")}
        </Button>
      </form>
    </Form>
  );
};
