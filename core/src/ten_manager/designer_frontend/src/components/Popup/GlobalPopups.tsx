import * as React from "react";

import { useWidgetStore } from "@/store/widget";
import { EWidgetCategory, EWidgetDisplayType } from "@/types/widgets";
import TerminalPopup from "@/components/Popup/TerminalPopup";
import EditorPopup from "@/components/Popup/EditorPopup";
import CustomNodeConnPopup from "@/components/Popup/CustomNodeConnPopup";

export function GlobalPopups() {
  const { widgets, removeWidget } = useWidgetStore();

  const [
    ,
    editorWidgetsMemo,
    terminalWidgetsMemo,
    customConnectionWidgetsMemo,
  ] = React.useMemo(() => {
    const popupWidgets = widgets.filter(
      (widget) => widget.display_type === EWidgetDisplayType.Popup
    );
    const editorWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Editor
    );
    const terminalWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Terminal
    );
    const customConnectionWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.CustomConnection
    );
    return [
      popupWidgets,
      editorWidgets,
      terminalWidgets,
      customConnectionWidgets,
    ];
  }, [widgets]);

  return (
    <>
      {terminalWidgetsMemo.map((widget) => (
        <TerminalPopup
          id={widget.id}
          key={`TerminalPopup-${widget.id}`}
          data={widget.metadata}
          onClose={() => removeWidget(widget.id)}
        />
      ))}
      {editorWidgetsMemo.map((widget) => (
        <EditorPopup
          id={widget.id}
          key={`EditorPopup-${widget.id}`}
          data={widget.metadata}
          onClose={() => removeWidget(widget.id)}
          hasUnsavedChanges={widget.isEditing}
        />
      ))}
      {customConnectionWidgetsMemo.map((widget) => (
        <CustomNodeConnPopup
          id={widget.id}
          key={`CustomNodeConnPopup-${widget.id}`}
          source={widget.metadata.source}
          target={widget.metadata.target}
          onClose={() => removeWidget(widget.id)}
        />
      ))}
    </>
  );
}
