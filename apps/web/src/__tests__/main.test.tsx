import { screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

describe("main", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    document.body.replaceChildren();
  });

  it("renders the home route into #root", async () => {
    const container = document.createElement("div");
    container.id = "root";
    document.body.append(container);

    await import("../main");

    expect(
      await screen.findByRole("heading", { level: 1, name: "Clinicore" }),
    ).toBeDefined();
  });

  it("fails loudly when index.html has no #root", async () => {
    await expect(import("../main")).rejects.toThrow(
      "The #root container is missing from index.html",
    );
  });
});
