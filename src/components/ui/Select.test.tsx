import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
import { Select } from "./Select";

afterEach(cleanup);

describe("Select", () => {
  it("渲染 rr-select 类与选项", () => {
    const { container } = render(
      <Select defaultValue="b">
        <option value="a">甲</option>
        <option value="b">乙</option>
      </Select>,
    );
    const el = container.querySelector("select")!;
    expect(el.classList.contains("rr-select")).toBe(true);
    expect(el.value).toBe("b");
    expect(el.querySelectorAll("option").length).toBe(2);
  });

  it("透传 className", () => {
    const { container } = render(
      <Select className="extra-class">
        <option value="a">甲</option>
      </Select>,
    );
    expect(container.querySelector("select")!.classList.contains("extra-class")).toBe(true);
  });
});
