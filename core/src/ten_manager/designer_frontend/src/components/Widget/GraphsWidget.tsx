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
import { ZodProvider } from "@autoform/zod";

import {
  AddNodePayloadSchema,
  AddConnectionPayloadSchema,
  EConnectionType,
  UpdateNodePropertyPayloadSchema,
  IGraph,
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
import { Textarea } from "@/components/ui/Textarea";
import { SpinnerLoading } from "@/components/Status/Loading";
import { Combobox } from "@/components/ui/Combobox";
import {
  useGraphs,
  postAddNode,
  retrieveGraphNodes,
  retrieveGraphConnections,
  postAddConnection,
  postUpdateNodeProperty,
} from "@/api/services/graphs";
import { retrieveExtensionPropertySchema } from "@/api/services/extension";
import { useAddons } from "@/api/services/addons";
import {
  generateRawNodes,
  generateRawEdges,
  updateNodesWithConnections,
  updateNodesWithAddonInfo,
  generateNodesAndEdges,
} from "@/flow/graph";
import { useAppStore, useFlowStore } from "@/store";
import type { TCustomNode } from "@/types/flow";
import { AutoForm } from "@/components/ui/autoform";
// eslint-disable-next-line max-len
import { convertExtensionPropertySchema2ZodSchema } from "@/components/Widget/utils";
import { cn } from "@/lib/utils";

// eslint-disable-next-line react-refresh/only-export-components
export const resetNodesAndEdgesByGraph = async (graph: IGraph) => {
  const backendNodes = await retrieveGraphNodes(graph.uuid);
  const backendConnections = await retrieveGraphConnections(graph.uuid);
  const rawNodes = generateRawNodes(backendNodes);
  const [rawEdges, rawEdgeAddressMap] = generateRawEdges(backendConnections);
  const nodesWithConnections = updateNodesWithConnections(
    rawNodes,
    rawEdgeAddressMap
  );
  const nodesWithAddonInfo = await updateNodesWithAddonInfo(
    graph.base_dir,
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
  graph_id?: string;
  postAddNodeActions?: () => void | Promise<void>;
}) => {
  const { base_dir, graph_id, postAddNodeActions } = props;
  const [customAddon, setCustomAddon] = React.useState<string | undefined>(
    undefined
  );
  const [isSubmitting, setIsSubmitting] = React.useState(false);
  const [remoteCheckErrorMessage, setRemoteCheckErrorMessage] = React.useState<
    string | undefined
  >(undefined);

  const { t } = useTranslation();
  const { currentWorkspace } = useAppStore();
  const { setNodesAndEdges } = useFlowStore();

  const form = useForm<z.infer<typeof AddNodePayloadSchema>>({
    resolver: zodResolver(AddNodePayloadSchema),
    defaultValues: {
      graph_id: graph_id ?? currentWorkspace?.graph?.uuid ?? "",
      node_name: undefined,
      addon_name: undefined,
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
      if (currentWorkspace?.graph?.uuid === data.graph_id) {
        const { nodes, edges } = await resetNodesAndEdgesByGraph(
          currentWorkspace.graph
        );
        setNodesAndEdges(nodes, edges);
        postAddNodeActions?.();
      }
      toast.success(t("popup.graph.addNodeSuccess"), {
        description: `Node ${data.node_name} added successfully`,
      });
    } catch (error) {
      console.error(error);
      setRemoteCheckErrorMessage(
        error instanceof Error ? error.message : "Unknown error"
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const { graphs, isLoading: isGraphsLoading, error: graphError } = useGraphs();
  const {
    addons,
    isLoading: isAddonsLoading,
    error: addonError,
  } = useAddons(base_dir);

  const comboboxOptionsMemo = React.useMemo(() => {
    const addonsOptions = addons
      ? addons.map((addon) => ({
          value: addon.name,
          label: addon.name,
        }))
      : [];
    const customAddons = customAddon
      ? [{ value: customAddon, label: customAddon }]
      : [];
    return [...addonsOptions, ...customAddons];
  }, [addons, customAddon]);

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
          name="graph_id"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("popup.graph.graphId")}</FormLabel>
              <FormControl>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <SelectTrigger className="w-full" disabled={isGraphsLoading}>
                    <SelectValue placeholder={t("popup.graph.graphId")} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>{t("popup.graph.graphId")}</SelectLabel>
                      {isGraphsLoading ? (
                        <SelectItem value={t("popup.graph.graphId")}>
                          <SpinnerLoading className="size-4" />
                        </SelectItem>
                      ) : (
                        graphs?.map((graph) => (
                          <SelectItem key={graph.uuid} value={graph.uuid}>
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
                <Combobox
                  options={comboboxOptionsMemo}
                  placeholder={t("popup.graph.addonName")}
                  selected={field.value}
                  onChange={(i) => {
                    field.onChange(i.value);
                  }}
                  onCreate={(i) => {
                    setCustomAddon(i);
                    field.onChange(i);
                  }}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

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

        {remoteCheckErrorMessage && (
          <div className="text-red-500">{remoteCheckErrorMessage}</div>
        )}

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

export const GraphAddConnectionWidget = (props: {
  base_dir: string;
  app_uri?: string | null;
  graph_id?: string;
  src_extension?: string;
  postAddConnectionActions?: () => void | Promise<void>;
}) => {
  const {
    base_dir,
    app_uri,
    graph_id,
    src_extension,
    postAddConnectionActions,
  } = props;
  const [isSubmitting, setIsSubmitting] = React.useState(false);

  const { t } = useTranslation();
  const { nodes, setNodesAndEdges } = useFlowStore();
  const { graphs, isLoading: isGraphsLoading, error: graphError } = useGraphs();
  const { currentWorkspace } = useAppStore();

  const form = useForm<z.infer<typeof AddConnectionPayloadSchema>>({
    resolver: zodResolver(AddConnectionPayloadSchema),
    defaultValues: {
      graph_id: graph_id ?? currentWorkspace?.graph?.uuid ?? "",
      src_app: app_uri,
      src_extension: src_extension ?? undefined,
      msg_type: EConnectionType.CMD,
      msg_name: undefined,
      dest_app: app_uri,
      dest_extension: undefined,
    },
  });

  const onSubmit = async (data: z.infer<typeof AddConnectionPayloadSchema>) => {
    setIsSubmitting(true);
    try {
      const payload = AddConnectionPayloadSchema.parse(data);
      if (payload.src_extension === payload.dest_extension) {
        throw new Error(t("popup.graph.sameNodeError"));
      }
      console.log("GraphAddConnection", payload);
      await postAddConnection(payload);
      if (
        currentWorkspace?.graph?.uuid === data.graph_id &&
        currentWorkspace?.app?.base_dir === base_dir
      ) {
        const { nodes, edges } = await resetNodesAndEdgesByGraph(
          currentWorkspace.graph
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
          name="graph_id"
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
                          <SelectItem key={graph.uuid} value={graph.uuid}>
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
                <Select
                  onValueChange={field.onChange}
                  value={field.value}
                  disabled={!!src_extension}
                >
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

export const GraphUpdateNodePropertyWidget = (props: {
  base_dir: string;
  app_uri?: string | null;
  graph_id?: string;
  node: TCustomNode;
  postUpdateNodePropertyActions?: () => void | Promise<void>;
}) => {
  const { app_uri, graph_id, node, postUpdateNodePropertyActions } = props;
  const [isSubmitting, setIsSubmitting] = React.useState(false);
  const [isSchemaLoading, setIsSchemaLoading] = React.useState(false);
  const [propertySchemaEntries, setPropertySchemaEntries] = React.useState<
    [string, z.ZodType][]
  >([]);

  const { t } = useTranslation();

  const { setNodesAndEdges } = useFlowStore();
  const { currentWorkspace } = useAppStore();

  React.useEffect(() => {
    const fetchSchema = async () => {
      try {
        setIsSchemaLoading(true);
        const schema = await retrieveExtensionPropertySchema({
          appBaseDir: currentWorkspace?.app?.base_dir ?? "",
          addonName: node.data.addon,
        });
        const propertySchemaEntries = convertExtensionPropertySchema2ZodSchema(
          schema.property_schema
        );
        setPropertySchemaEntries(propertySchemaEntries);
      } catch (error) {
        console.error(error);
        toast.error(error instanceof Error ? error.message : "Unknown error");
      } finally {
        setIsSchemaLoading(false);
      }
    };

    fetchSchema();
  }, [currentWorkspace?.app?.base_dir, node.data.addon]);

  console.log(node?.data.property);

  return (
    <>
      {isSchemaLoading && !propertySchemaEntries && (
        <SpinnerLoading className="size-4" />
      )}
      {propertySchemaEntries?.length > 0 ? (
        <AutoForm
          values={node?.data.property || {}}
          schema={
            new ZodProvider(z.object(Object.fromEntries(propertySchemaEntries)))
          }
          onSubmit={async (data) => {
            console.log(data);
            setIsSubmitting(true);
            try {
              const nodeData = UpdateNodePropertyPayloadSchema.parse({
                graph_id: graph_id ?? currentWorkspace?.graph?.uuid ?? "",
                node_name: node.data.name,
                addon_name: node.data.addon,
                extension_group_name: node.data.extension_group,
                app_uri: app_uri ?? undefined,
                property: JSON.stringify(data, null, 2),
              });
              await postUpdateNodeProperty(nodeData);
              if (currentWorkspace?.graph) {
                const { nodes, edges } = await resetNodesAndEdgesByGraph(
                  currentWorkspace.graph
                );
                setNodesAndEdges(nodes, edges);
              }
              toast.success(t("popup.graph.updateNodePropertySuccess"), {
                description: `${node.data.name}`,
              });
              postUpdateNodePropertyActions?.();
            } catch (error) {
              console.error(error);
              toast.error(t("popup.graph.updateNodePropertyFailed"), {
                description:
                  error instanceof Error ? error.message : "Unknown error",
              });
            } finally {
              setIsSubmitting(false);
            }
          }}
          withSubmit
          formProps={{
            className: cn(
              "flex flex-col gap-4 overflow-y-auto px-1 h-full",
              isSubmitting && "disabled"
            ),
          }}
        />
      ) : (
        <div className="text-center text-sm text-gray-500">
          {t("popup.graph.noPropertySchema")}
        </div>
      )}
    </>
  );
};
