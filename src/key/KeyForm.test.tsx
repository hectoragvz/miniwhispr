import { fireEvent, render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { KeyForm } from "./KeyForm";

function keyField(): HTMLInputElement {
  return screen.getByLabelText("Groq API key");
}

function saveButton(): HTMLButtonElement {
  return screen.getByRole("button", { name: "Save" });
}

test("Save stays disabled until a key is typed", () => {
  render(<KeyForm hasKey={false} onSave={vi.fn()} />);
  expect(saveButton().disabled).toBe(true);

  fireEvent.change(keyField(), { target: { value: "   " } });
  expect(saveButton().disabled).toBe(true);

  fireEvent.change(keyField(), { target: { value: "gsk_abc" } });
  expect(saveButton().disabled).toBe(false);
});

test("saving sends the key without surrounding spaces", () => {
  const onSave = vi.fn(() => Promise.resolve());
  render(<KeyForm hasKey={false} onSave={onSave} />);

  fireEvent.change(keyField(), { target: { value: "  gsk_abc \n" } });
  fireEvent.click(saveButton());

  expect(onSave).toHaveBeenCalledWith("gsk_abc");
});

test("the key field hides what is typed", () => {
  render(<KeyForm hasKey={false} onSave={vi.fn()} />);

  expect(keyField().type).toBe("password");
});

test("with a saved key, the field asks for a replacement", () => {
  render(<KeyForm hasKey={true} onSave={vi.fn()} />);

  expect(keyField().placeholder).toBe("Paste a new key to replace the saved one");
  expect(keyField().value).toBe("");
});

test("a failed save says why and keeps the key typed", async () => {
  const onSave = vi.fn(() => Promise.reject("Could not save the Groq API key to the Keychain"));
  render(<KeyForm hasKey={false} onSave={onSave} />);

  fireEvent.change(keyField(), { target: { value: "gsk_abc" } });
  fireEvent.click(saveButton());

  expect((await screen.findByRole("alert")).textContent).toBe(
    "Could not save the Groq API key to the Keychain",
  );
  expect(keyField().value).toBe("gsk_abc");
});
