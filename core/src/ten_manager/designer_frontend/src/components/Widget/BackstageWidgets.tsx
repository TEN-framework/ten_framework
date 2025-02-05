import { LogViewerBackstageWidget } from "@/components/Widget/LogViewerWidget";
import { useWidgetStore } from "@/store/widget";
import { EWidgetCategory } from "@/types/widgets";

export function BackstageWidgets() {
  const { backstageWidgets } = useWidgetStore();

  return (
    <>
      {backstageWidgets.map((widget) => {
        switch (widget.category) {
          case EWidgetCategory.LogViewer:
            return <LogViewerBackstageWidget key={widget.id} {...widget} />;
          default:
            return null;
        }
      })}
    </>
  );
}
