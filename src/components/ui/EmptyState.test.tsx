import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup, screen } from "@testing-library/react";
import i18n from "@/i18n";
import { EmptyState } from "./EmptyState";

afterEach(cleanup);

describe("EmptyState", () => {
  it("渲染 rr-empty 类与默认 i18n 标题", () => {
    const { container } = render(<EmptyState />);
    const el = container.firstElementChild as HTMLElement;
    expect(el.classList.contains("rr-empty")).toBe(true);
    expect(screen.getByText(i18n.t("ui.empty.title"))).not.toBeNull();
  });

  it("自定义标题/描述/操作插槽与 className 透传", () => {
    const { container } = render(
      <EmptyState
        className="extra-class"
        title="库为空"
        description="添加目录开始"
        action={<button>添加</button>}
      />,
    );
    const el = container.firstElementChild as HTMLElement;
    expect(el.classList.contains("extra-class")).toBe(true);
    expect(screen.getByText("库为空")).not.toBeNull();
    expect(screen.getByText("添加目录开始")).not.toBeNull();
    expect(screen.getByText("添加")).not.toBeNull();
  });
});
