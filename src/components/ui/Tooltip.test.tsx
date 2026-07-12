import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup, screen } from "@testing-library/react";
import { Tooltip } from "./Tooltip";

afterEach(cleanup);

describe("Tooltip", () => {
  it("渲染 rr-tooltip 类与提示内容", () => {
    const { container } = render(
      <Tooltip content="删除该项">
        <button>x</button>
      </Tooltip>,
    );
    const root = container.querySelector(".rr-tooltip") as HTMLElement;
    expect(root).not.toBeNull();
    const tip = screen.getByRole("tooltip");
    expect(tip.textContent).toBe("删除该项");
    expect(tip.className).toContain("group-hover:opacity-100");
  });

  it("side 变体差异与 className 透传", () => {
    const { container: top } = render(
      <Tooltip content="t" className="extra-class">
        <span>a</span>
      </Tooltip>,
    );
    const { container: bottom } = render(
      <Tooltip content="t" side="bottom">
        <span>b</span>
      </Tooltip>,
    );
    expect(top.querySelector(".rr-tooltip")!.classList.contains("extra-class")).toBe(true);
    expect(top.querySelector("[role=tooltip]")!.className).toContain("bottom-full");
    expect(bottom.querySelector("[role=tooltip]")!.className).toContain("top-full");
  });
});
