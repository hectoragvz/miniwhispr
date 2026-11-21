import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Toast, type ToastState } from "./Toast";

/** The Toast window's page: shows the states the Dictation core sends. */
export function ToastWindow() {
  const [state, setState] = useState<ToastState>({ kind: "hidden" });

  useEffect(() => {
    const unlisten = getCurrentWebviewWindow().listen<ToastState>("toast", (event) =>
      setState(event.payload),
    );
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  return <Toast state={state} onHidden={() => void invoke("toast_hidden")} />;
}
