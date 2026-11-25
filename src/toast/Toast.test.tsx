import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { Toast } from "./Toast";

const noop = () => {};

function barHeights(): number[] {
  return screen.getAllByTestId("wave-bar").map((bar) => parseFloat(bar.style.height));
}

test("listening shows voice waves that rise with the microphone level", () => {
  const { rerender } = render(<Toast state={{ kind: "listening", level: 0 }} onHidden={noop} />);
  const quiet = barHeights();

  rerender(<Toast state={{ kind: "listening", level: 1 }} onHidden={noop} />);
  const loud = barHeights();

  expect(screen.getByRole("status", { name: "Listening" })).toBeTruthy();
  expect(quiet.length).toBeGreaterThan(0);
  loud.forEach((height, i) => expect(height).toBeGreaterThan(quiet[i]));
  expect(screen.getByRole("status").textContent).toBe("");
});

test("processing shows a spinner instead of the waves", () => {
  render(<Toast state={{ kind: "processing" }} onHidden={noop} />);

  const toast = screen.getByRole("status", { name: "Processing" });
  expect(screen.getByTestId("spinner")).toBeTruthy();
  expect(screen.queryAllByTestId("wave-bar")).toHaveLength(0);
  expect(toast.textContent).toBe("");
});

test("hidden shows nothing", () => {
  const { container } = render(<Toast state={{ kind: "hidden" }} onHidden={noop} />);

  expect(container.innerHTML).toBe("");
});

test("hiding animates the Toast out, then reports it is gone", () => {
  const onHidden = vi.fn();
  const { container, rerender } = render(
    <Toast state={{ kind: "processing" }} onHidden={onHidden} />,
  );

  rerender(<Toast state={{ kind: "hidden" }} onHidden={onHidden} />);
  const leaving = screen.getByRole("status");
  expect(onHidden).not.toHaveBeenCalled();

  fireEvent.animationEnd(leaving);

  expect(onHidden).toHaveBeenCalledOnce();
  expect(container.innerHTML).toBe("");
});

test("listening again while animating out keeps the Toast", () => {
  const onHidden = vi.fn();
  const { rerender } = render(<Toast state={{ kind: "processing" }} onHidden={onHidden} />);
  rerender(<Toast state={{ kind: "hidden" }} onHidden={onHidden} />);

  rerender(<Toast state={{ kind: "listening", level: 0 }} onHidden={onHidden} />);
  fireEvent.animationEnd(screen.getByRole("status"));

  expect(onHidden).not.toHaveBeenCalled();
  expect(screen.getByRole("status", { name: "Listening" })).toBeTruthy();
});

test("an error flashes the Toast red and shakes it, then it hides, with no text", () => {
  const onHidden = vi.fn();
  const { container, rerender } = render(
    <Toast state={{ kind: "processing" }} onHidden={onHidden} />,
  );

  rerender(<Toast state={{ kind: "error" }} onHidden={onHidden} />);
  const toast = screen.getByRole("status", { name: "Error" });
  expect(toast.classList).toContain("toast--error");
  expect(screen.queryByTestId("spinner")).toBeNull();
  expect(toast.textContent).toBe("");

  // The shake ending is not the Toast leaving.
  fireEvent.animationEnd(toast);
  expect(onHidden).not.toHaveBeenCalled();

  rerender(<Toast state={{ kind: "hidden" }} onHidden={onHidden} />);
  expect(screen.getByRole("status", { name: "Error" }).classList).toContain("toast--error");
  fireEvent.animationEnd(screen.getByRole("status"));

  expect(onHidden).toHaveBeenCalledOnce();
  expect(container.innerHTML).toBe("");
});
