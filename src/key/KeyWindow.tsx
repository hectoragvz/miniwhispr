import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { KeyForm } from "./KeyForm";

/**
 * The Groq API key window's page. The window opens hidden; once the page is
 * laid out it asks to be sized to fit and shown. Saving closes the window.
 */
export function KeyWindow() {
  const [hasKey, setHasKey] = useState(false);
  const page = useRef<HTMLDivElement>(null);
  const shown = useRef(false);

  useEffect(() => {
    invoke<boolean>("has_groq_key").then(setHasKey, (reason: unknown) =>
      console.error("Could not tell whether a Groq API key is saved:", reason),
    );
  }, []);

  useLayoutEffect(() => {
    // Once only: React runs effects twice in development.
    if (shown.current || page.current === null) {
      return;
    }
    shown.current = true;
    invoke("show_key_window", {
      contentHeight: page.current.getBoundingClientRect().height,
      visibleHeight: window.innerHeight,
    }).catch((reason: unknown) => console.error("Could not show the window:", reason));
  }, []);

  return (
    <div ref={page}>
      <KeyForm hasKey={hasKey} onSave={(key) => invoke("save_groq_key", { key })} />
    </div>
  );
}
