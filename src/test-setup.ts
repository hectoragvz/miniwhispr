import { afterEach } from "vitest";

// jsdom has no AnimationEvent, so React would listen for the old
// `webkitAnimationEnd` name instead of `animationend`, which is what WebKit
// fires in the real Toast window. A stand-in makes React use the real name.
if (typeof window.AnimationEvent === "undefined") {
  window.AnimationEvent = class extends Event {
    readonly animationName = "";
    readonly elapsedTime = 0;
    readonly pseudoElement = "";
  };
}

// Imported only now so that React sees the stand-in.
const { cleanup } = await import("@testing-library/react");

// Testing Library only unmounts between tests by itself when Vitest globals
// are on; they are off here, so do it explicitly.
afterEach(cleanup);
