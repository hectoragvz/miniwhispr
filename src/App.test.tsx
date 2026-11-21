import { render } from "@testing-library/react";
import { expect, test } from "vitest";
import App from "./App";

test("renders nothing until there is a window to show", () => {
  const { container } = render(<App />);

  expect(container.innerHTML).toBe("");
});
