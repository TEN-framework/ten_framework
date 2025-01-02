export const dispatchCustomNodeActionPopup = (
  action: string,
  source: string,
  target?: string
) => {
  if (typeof window !== "undefined") {
    window.dispatchEvent(
      new CustomEvent("customNodeAction", {
        detail: { action, source, target },
      })
    );
  }
};
