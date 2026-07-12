import { describe, it, expect, afterEach, vi } from "vitest";
import { render, cleanup, fireEvent, screen } from "@testing-library/react";
import { Tabs } from "./Tabs";

afterEach(cleanup);

const items = [
  { value: "a", label: "外观" },
  { value: "b", label: "通用" },
];

describe("Tabs", () => {
  it("渲染 rr-tabs 类与选中态差异", () => {
    const { container } = render(<Tabs items={items} value="a" onChange={() => {}} />);
    const root = container.querySelector(".rr-tabs") as HTMLElement;
    expect(root).not.toBeNull();
    const tabs = root.querySelectorAll("[role=tab]");
    expect(tabs.length).toBe(2);
    expect(tabs[0].getAttribute("aria-selected")).toBe("true");
    expect(tabs[0].className).toContain("bg-accent-primary");
    expect(tabs[1].getAttribute("aria-selected")).toBe("false");
    expect(tabs[1].className).not.toContain("bg-accent-primary");
  });

  it("点击触发 onChange", () => {
    const onChange = vi.fn();
    render(<Tabs items={items} value="a" onChange={onChange} />);
    fireEvent.click(screen.getByText("通用"));
    expect(onChange).toHaveBeenCalledWith("b");
  });

  it("透传 className", () => {
    const { container } = render(
      <Tabs items={items} value="a" onChange={() => {}} className="extra-class" />,
    );
    expect(container.querySelector(".rr-tabs")!.classList.contains("extra-class")).toBe(true);
  });
});
