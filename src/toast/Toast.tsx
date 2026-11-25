import { useState, type AnimationEvent } from "react";
import "./Toast.css";

/** What the Toast shows. Mirrors `Toast` in `src-tauri/src/dictation.rs`. */
export type ToastState =
  | { kind: "hidden" }
  | { kind: "listening"; level: number }
  | { kind: "processing" }
  | { kind: "error" };

/** How much each voice wave bar follows the level, centre bars most. */
const BAR_WEIGHTS = [0.45, 0.7, 0.9, 1, 0.9, 0.7, 0.45];
const BAR_MIN_PX = 3;
const BAR_MAX_PX = 18;

interface Props {
  state: ToastState;
  onHidden: () => void;
}

type Shown = Exclude<ToastState, { kind: "hidden" }>;

const LABELS: Record<Shown["kind"], string> = {
  listening: "Listening",
  processing: "Processing",
  error: "Error",
};

/**
 * The text-free pill shown during a Dictation. When `state` becomes hidden it
 * keeps showing what it last showed while it animates out, then calls
 * `onHidden`.
 */
export function Toast({ state, onHidden }: Props) {
  const [last, setLast] = useState<Shown | null>(null);
  if (state.kind !== "hidden" && state !== last) {
    setLast(state);
  }
  const shown = state.kind === "hidden" ? last : state;
  if (shown === null) {
    return null;
  }

  const leaving = state.kind === "hidden";
  const className = ["toast", shown.kind === "error" && "toast--error", leaving && "toast--leaving"]
    .filter(Boolean)
    .join(" ");
  const onAnimationEnd = (event: AnimationEvent) => {
    // Only the pill's own exit animation, not its waves, spinner or shake.
    if (leaving && event.target === event.currentTarget) {
      setLast(null);
      onHidden();
    }
  };
  return (
    <div
      className={className}
      role="status"
      aria-label={LABELS[shown.kind]}
      onAnimationEnd={onAnimationEnd}
    >
      {shown.kind === "listening" && <Waves level={shown.level} />}
      {shown.kind === "processing" && <div className="spinner" data-testid="spinner" />}
    </div>
  );
}

function Waves({ level }: { level: number }) {
  return (
    <div className="waves">
      {BAR_WEIGHTS.map((weight, i) => (
        <span
          key={i}
          className="wave-bar"
          data-testid="wave-bar"
          style={{ height: `${BAR_MIN_PX + (BAR_MAX_PX - BAR_MIN_PX) * level * weight}px` }}
        />
      ))}
    </div>
  );
}
