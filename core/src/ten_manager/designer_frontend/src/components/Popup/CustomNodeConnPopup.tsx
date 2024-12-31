import * as React from "react";

import Popup from "@/components/Popup/Popup";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

export interface CustomNodeConnPopupProps {
  source: string;
  target?: string;
  onClose?: () => void;
}

const CustomNodeConnPopup: React.FC<CustomNodeConnPopupProps> = ({
  source,
  target,
  onClose,
}) => {
  return (
    <Popup
      title="Custom Node Connection"
      onClose={() => onClose?.()}
      initialWidth={DEFAULT_WIDTH}
      initialHeight={DEFAULT_HEIGHT}
      resizable
    >
      <div>
        <p>Source: {source}</p>
        <p>Target: {target}</p>
      </div>
    </Popup>
  );
};

export default CustomNodeConnPopup;
