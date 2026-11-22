import { useState, type FormEvent } from "react";
import "./KeyForm.css";

interface Props {
  /** Whether a key is already saved. The saved key itself is never shown. */
  hasKey: boolean;
  /** Saves the key. Rejects with the reason it could not be saved. */
  onSave: (key: string) => Promise<void>;
}

/** Enters or replaces the Groq API key. */
export function KeyForm({ hasKey, onSave }: Props) {
  const [key, setKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const trimmed = key.trim();

  const save = (event: FormEvent) => {
    event.preventDefault();
    setSaving(true);
    setError(null);
    onSave(trimmed).catch((reason: unknown) => {
      setError(String(reason));
      setSaving(false);
    });
  };

  return (
    <form className="key-form" onSubmit={save}>
      <p className="key-intro">
        Miniwhispr turns your speech into text with Groq. Paste your key from
        console.groq.com/keys.
      </p>
      <input
        className="key-field"
        type="password"
        aria-label="Groq API key"
        placeholder={hasKey ? "Paste a new key to replace the saved one" : "gsk_…"}
        value={key}
        onChange={(event) => setKey(event.target.value)}
        autoFocus
        autoComplete="off"
        spellCheck={false}
      />
      <div className="key-footer">
        {error === null ? (
          <span className="key-hint">Kept in your macOS Keychain.</span>
        ) : (
          <span className="key-error" role="alert" title={error}>
            {error}
          </span>
        )}
        <button className="key-save" type="submit" disabled={trimmed === "" || saving}>
          Save
        </button>
      </div>
    </form>
  );
}
