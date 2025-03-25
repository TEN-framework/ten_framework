//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { Popup } from "@/components/Popup/Popup";
import { Separator } from "@/components/ui/Separator";
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
import { Input } from "@/components/ui/Input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/Tabs";
import { cn } from "@/lib/utils";
import { PREFERENCES_POPUP_ID } from "@/constants/widgets";
import { useAppStore, useWidgetStore } from "@/store";
import {
  PreferencesSchema,
  PreferencesLogSchema,
  EPreferencesTabs,
} from "@/types/apps";
import { getZodDefaults } from "@/utils";

export const PreferencesPopup = () => {
  const { t } = useTranslation();
  const { preferences, setPreferences } = useAppStore();
  const { removeWidget } = useWidgetStore();

  const onClose = () => {
    removeWidget(PREFERENCES_POPUP_ID);
  };

  return (
    <Popup
      id={PREFERENCES_POPUP_ID}
      title={t("header.menuDesigner.preferences")}
    >
      <Tabs defaultValue={EPreferencesTabs.GENERAL} className="w-full">
        <TabsList className={cn("grid w-full grid-cols-2 min-w-[400px]")}>
          <TabsTrigger value={EPreferencesTabs.GENERAL}>
            {t("preferences.general.title")}
          </TabsTrigger>
          <TabsTrigger value={EPreferencesTabs.LOG}>
            {t("preferences.log.title")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value={EPreferencesTabs.GENERAL}>
          <PreferencesGeneralTab />
        </TabsContent>
        <TabsContent value={EPreferencesTabs.LOG}>
          <PreferencesLogTab onCancel={onClose} />
        </TabsContent>
      </Tabs>
    </Popup>
  );
};

export const PreferencesGeneralTab = () => {
  return <div>PreferencesGeneralTab</div>;
};

export const PreferencesLogTab = (props: {
  onCancel?: () => void;
  onSave?: (values: z.infer<typeof PreferencesLogSchema>) => void;
}) => {
  const { t } = useTranslation();
  const { preferences, setPreferences } = useAppStore();
  const form = useForm<z.infer<typeof PreferencesLogSchema>>({
    resolver: zodResolver(PreferencesLogSchema),
    defaultValues: preferences[EPreferencesTabs.LOG],
  });

  const onSubmit = (values: z.infer<typeof PreferencesLogSchema>) => {
    // Do something with the form values.
    // ✅ This will be type-safe and validated.
    setPreferences(EPreferencesTabs.LOG, values);
    props.onSave?.(values);
    toast.success(t("preferences.saved"), {
      description: t("preferences.savedSuccess", { key: EPreferencesTabs.LOG }),
    });
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="maxLines"
          render={({ field }) => (
            <FormItem>
              <FormLabel>{t("preferences.log.maxLines")}</FormLabel>
              <FormControl>
                <Input type="number" {...field} />
              </FormControl>
              <FormDescription>
                {t("preferences.log.maxLinesDescription")}
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={props.onCancel}>
            {t("preferences.cancel")}
          </Button>
          <Button type="submit">{t("preferences.save")}</Button>
        </div>
      </form>
    </Form>
  );
};
