#!/usr/bin/env python3
"""Generate the formal SWTC preliminary-round report PDF."""

from __future__ import annotations

from pathlib import Path

from PIL import Image as PILImage
from reportlab.graphics.shapes import Drawing, Line, Polygon, Rect, String
from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import cm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    Image,
    KeepTogether,
    ListFlowable,
    ListItem,
    PageBreak,
    PageTemplate,
    Paragraph,
    Preformatted,
    Spacer,
    Table,
    TableStyle,
)
from reportlab.platypus.tableofcontents import TableOfContents


REPO_ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = REPO_ROOT / "docs" / "assets"
OUTPUT_PDF = REPO_ROOT / "SWTC-初赛文档.pdf"

PAGE_WIDTH, PAGE_HEIGHT = A4
MARGIN_X = 1.8 * cm
MARGIN_TOP = 1.8 * cm
MARGIN_BOTTOM = 1.65 * cm

RED = colors.HexColor("#B51F2E")
DEEP_RED = colors.HexColor("#8E1622")
NAVY = colors.HexColor("#17365D")
BLUE = colors.HexColor("#2F75B5")
TEAL = colors.HexColor("#2A8C82")
PALE_BLUE = colors.HexColor("#EAF2F8")
PALE_RED = colors.HexColor("#F9ECEE")
PALE_GRAY = colors.HexColor("#F4F6F8")
MID_GRAY = colors.HexColor("#66717F")
LIGHT_LINE = colors.HexColor("#D6DCE4")
BLACK = colors.black
TEXT = BLACK


def register_fonts() -> None:
    font_dir = Path("C:/Windows/Fonts")
    regular = font_dir / "Deng.ttf"
    bold = font_dir / "Dengb.ttf"
    if not regular.exists() or not bold.exists():
        regular = font_dir / "simhei.ttf"
        bold = regular
    pdfmetrics.registerFont(TTFont("SWTCCN", str(regular)))
    pdfmetrics.registerFont(TTFont("SWTCCN-Bold", str(bold)))
    pdfmetrics.registerFontFamily(
        "SWTCCN",
        normal="SWTCCN",
        bold="SWTCCN-Bold",
        italic="SWTCCN",
        boldItalic="SWTCCN-Bold",
    )


def build_styles() -> dict[str, ParagraphStyle]:
    base = getSampleStyleSheet()
    return {
        "cover_comp": ParagraphStyle(
            "CoverCompetition",
            parent=base["Normal"],
            fontName="SWTCCN",
            fontSize=15,
            leading=23,
            alignment=TA_CENTER,
            textColor=BLACK,
        ),
        "cover_title": ParagraphStyle(
            "CoverTitle",
            parent=base["Title"],
            fontName="SWTCCN-Bold",
            fontSize=29,
            leading=38,
            alignment=TA_CENTER,
            textColor=BLACK,
        ),
        "cover_subtitle": ParagraphStyle(
            "CoverSubtitle",
            parent=base["Normal"],
            fontName="SWTCCN",
            fontSize=13,
            leading=20,
            alignment=TA_CENTER,
            textColor=BLACK,
        ),
        "front_title": ParagraphStyle(
            "FrontTitle",
            parent=base["Title"],
            fontName="SWTCCN-Bold",
            fontSize=21,
            leading=30,
            alignment=TA_CENTER,
            textColor=BLACK,
            spaceAfter=18,
        ),
        "h1": ParagraphStyle(
            "H1",
            parent=base["Heading1"],
            fontName="SWTCCN-Bold",
            fontSize=19,
            leading=25,
            spaceBefore=0,
            spaceAfter=13,
            textColor=BLACK,
        ),
        "h2": ParagraphStyle(
            "H2",
            parent=base["Heading2"],
            fontName="SWTCCN-Bold",
            fontSize=13.5,
            leading=19,
            spaceBefore=8,
            spaceAfter=7,
            textColor=BLACK,
        ),
        "body": ParagraphStyle(
            "Body",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=10.2,
            leading=17,
            alignment=TA_LEFT,
            firstLineIndent=2 * 10.2,
            textColor=BLACK,
            spaceAfter=6,
        ),
        "body_no_indent": ParagraphStyle(
            "BodyNoIndent",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=10.2,
            leading=16,
            alignment=TA_LEFT,
            textColor=BLACK,
            spaceAfter=5,
        ),
        "small": ParagraphStyle(
            "Small",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=13,
            textColor=BLACK,
        ),
        "table": ParagraphStyle(
            "TableText",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=12.5,
            textColor=BLACK,
        ),
        "table_center": ParagraphStyle(
            "TableCenter",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=12.5,
            alignment=TA_CENTER,
            textColor=BLACK,
        ),
        "caption": ParagraphStyle(
            "Caption",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.8,
            leading=13,
            alignment=TA_CENTER,
            textColor=BLACK,
            spaceBefore=5,
            spaceAfter=8,
        ),
        "quote": ParagraphStyle(
            "Quote",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=10,
            leading=16,
            leftIndent=12,
            rightIndent=12,
            borderColor=RED,
            borderWidth=0,
            borderPadding=9,
            backColor=PALE_RED,
            textColor=BLACK,
            spaceBefore=7,
            spaceAfter=8,
        ),
        "reference": ParagraphStyle(
            "Reference",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=13,
            leftIndent=12,
            firstLineIndent=-12,
            textColor=BLACK,
            spaceAfter=4,
        ),
        "code": ParagraphStyle(
            "Code",
            parent=base["Code"],
            fontName="Courier",
            fontSize=7.8,
            leading=11.2,
            leftIndent=0,
            rightIndent=0,
            textColor=BLACK,
            backColor=colors.white,
            borderPadding=9,
            spaceBefore=5,
            spaceAfter=8,
        ),
        "toc_level0": ParagraphStyle(
            "TOCLevel0",
            fontName="SWTCCN-Bold",
            fontSize=11,
            leading=17,
            leftIndent=0,
            firstLineIndent=0,
            textColor=BLACK,
            spaceBefore=4,
        ),
        "toc_level1": ParagraphStyle(
            "TOCLevel1",
            fontName="SWTCCN",
            fontSize=9.2,
            leading=14,
            leftIndent=18,
            firstLineIndent=0,
            textColor=BLACK,
        ),
    }


def p(text: str, style: ParagraphStyle) -> Paragraph:
    return Paragraph(text, style)


def code_block(text: str, styles: dict[str, ParagraphStyle]) -> Table:
    block = Preformatted(text.strip("\n"), styles["code"])
    container = Table([[block]], colWidths=[16.8 * cm], hAlign="LEFT")
    container.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), colors.white),
                ("LEFTPADDING", (0, 0), (-1, -1), 10),
                ("RIGHTPADDING", (0, 0), (-1, -1), 10),
                ("TOPPADDING", (0, 0), (-1, -1), 8),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 8),
                ("BOX", (0, 0), (-1, -1), 0.5, NAVY),
            ]
        )
    )
    return container


def table_cell(text: str, styles: dict[str, ParagraphStyle], center: bool = False) -> Paragraph:
    return p(text, styles["table_center" if center else "table"])


def make_table(
    data: list[list[object]],
    col_widths: list[float],
    header: bool = True,
    compact: bool = False,
) -> Table:
    tbl = Table(data, colWidths=col_widths, repeatRows=1 if header else 0, hAlign="LEFT")
    commands: list[tuple] = [
        ("FONTNAME", (0, 0), (-1, -1), "SWTCCN"),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("GRID", (0, 0), (-1, -1), 0.45, LIGHT_LINE),
        ("LEFTPADDING", (0, 0), (-1, -1), 6),
        ("RIGHTPADDING", (0, 0), (-1, -1), 6),
        ("TOPPADDING", (0, 0), (-1, -1), 4 if compact else 6),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4 if compact else 6),
        ("ROWBACKGROUNDS", (0, 1 if header else 0), (-1, -1), [colors.white, PALE_GRAY]),
    ]
    if header:
        commands.extend(
            [
                ("BACKGROUND", (0, 0), (-1, 0), PALE_GRAY),
                ("TEXTCOLOR", (0, 0), (-1, 0), BLACK),
                ("FONTNAME", (0, 0), (-1, 0), "SWTCCN-Bold"),
                ("ALIGN", (0, 0), (-1, 0), "CENTER"),
            ]
        )
    tbl.setStyle(TableStyle(commands))
    return tbl


def bullets(items: list[str], styles: dict[str, ParagraphStyle]) -> ListFlowable:
    return ListFlowable(
        [ListItem(p(item, styles["body_no_indent"]), leftIndent=12) for item in items],
        bulletType="bullet",
        start="circle",
        leftIndent=18,
        bulletFontName="SWTCCN",
        bulletFontSize=7,
        bulletColor=RED,
        spaceAfter=5,
    )


def add_box(d: Drawing, x: float, y: float, w: float, h: float, lines: list[str], fill: colors.Color) -> None:
    d.add(Rect(x, y, w, h, rx=6, ry=6, fillColor=fill, strokeColor=LIGHT_LINE, strokeWidth=0.8))
    line_height = 11
    start_y = y + h / 2 + (len(lines) - 1) * line_height / 2 - 3
    for index, line in enumerate(lines):
        d.add(
            String(
                x + w / 2,
                start_y - index * line_height,
                line,
                fontName="SWTCCN-Bold" if index == 0 else "SWTCCN",
                fontSize=8.2,
                fillColor=BLACK,
                textAnchor="middle",
            )
        )


def add_arrow(d: Drawing, x1: float, y1: float, x2: float, y2: float, color: colors.Color = MID_GRAY) -> None:
    d.add(Line(x1, y1, x2, y2, strokeColor=color, strokeWidth=1.2))
    if abs(x2 - x1) >= abs(y2 - y1):
        direction = 1 if x2 >= x1 else -1
        d.add(Polygon([x2, y2, x2 - 5 * direction, y2 + 3, x2 - 5 * direction, y2 - 3], fillColor=color, strokeColor=color))
    else:
        direction = 1 if y2 >= y1 else -1
        d.add(Polygon([x2, y2, x2 - 3, y2 - 5 * direction, x2 + 3, y2 - 5 * direction], fillColor=color, strokeColor=color))


def architecture_diagram() -> Drawing:
    d = Drawing(17.2 * cm, 8.7 * cm)
    d.add(String(8, 237, "统一构建层", fontName="SWTCCN-Bold", fontSize=9, fillColor=BLACK))
    add_box(d, 20, 190, 105, 38, ["GitLab main", "统一源码入口"], PALE_BLUE)
    add_box(d, 176, 190, 105, 38, ["根目录 make all", "离线构建与门禁"], PALE_RED)
    add_box(d, 330, 211, 108, 34, ["kernel-rv", "RISC-V64 ELF"], colors.HexColor("#E8F3EE"))
    add_box(d, 330, 166, 108, 34, ["kernel-la", "LoongArch64 ELF"], colors.HexColor("#FFF3DF"))
    add_arrow(d, 125, 209, 176, 209)
    d.add(Line(281, 209, 305, 209, strokeColor=MID_GRAY, strokeWidth=1.2))
    d.add(Line(305, 183, 305, 228, strokeColor=MID_GRAY, strokeWidth=1.2))
    add_arrow(d, 305, 228, 330, 228)
    add_arrow(d, 305, 183, 330, 183)

    d.add(String(8, 145, "官方运行层", fontName="SWTCCN-Bold", fontSize=9, fillColor=BLACK))
    add_box(d, 20, 98, 92, 35, ["QEMU", "双架构启动"], PALE_BLUE)
    add_box(d, 141, 98, 92, 35, ["virtio-blk", "EXT4 测试盘"], PALE_BLUE)
    add_box(d, 262, 98, 92, 35, ["测试资产", "脚本与 ELF"], PALE_BLUE)
    add_box(d, 383, 98, 92, 35, ["官方输出", "计分与关机"], PALE_BLUE)
    add_arrow(d, 112, 115.5, 141, 115.5)
    add_arrow(d, 233, 115.5, 262, 115.5)
    add_arrow(d, 354, 115.5, 383, 115.5)
    d.add(Line(384, 166, 384, 151, strokeColor=MID_GRAY, strokeWidth=1.2))
    d.add(Line(66, 151, 384, 151, strokeColor=MID_GRAY, strokeWidth=1.2))
    add_arrow(d, 66, 151, 66, 133)

    d.add(String(8, 75, "内核运行路径", fontName="SWTCCN-Bold", fontSize=9, fillColor=BLACK))
    labels = [
        ["启动", "平台初始化"],
        ["存储", "EXT4 读取"],
        ["调度", "脚本队列"],
        ["装载", "ELF/运行时"],
        ["兼容", "Linux ABI"],
        ["收尾", "日志/退出"],
    ]
    fills = [PALE_GRAY, PALE_GRAY, PALE_GRAY, PALE_GRAY, PALE_GRAY, PALE_GRAY]
    start_x, y, w, h, gap = 20, 18, 68, 36, 14
    for i, label in enumerate(labels):
        x = start_x + i * (w + gap)
        add_box(d, x, y, w, h, label, fills[i])
        if i < len(labels) - 1:
            add_arrow(d, x + w, y + h / 2, x + w + gap, y + h / 2)
    return d


def pipeline_diagram(title: str, steps: list[tuple[str, str]], accent: colors.Color = BLUE) -> Drawing:
    width = 17.2 * cm
    height = 4.4 * cm
    d = Drawing(width, height)
    d.add(String(8, height - 17, title, fontName="SWTCCN-Bold", fontSize=10, fillColor=BLACK))
    gap = 16
    box_w = (width - 24 - gap * (len(steps) - 1)) / len(steps)
    box_h = 60
    y = 28
    for index, (heading, detail) in enumerate(steps):
        x = 12 + index * (box_w + gap)
        fill = PALE_RED if index == len(steps) - 1 else PALE_BLUE
        d.add(Rect(x, y, box_w, box_h, rx=7, ry=7, fillColor=fill, strokeColor=LIGHT_LINE, strokeWidth=0.8))
        d.add(String(x + box_w / 2, y + 37, heading, fontName="SWTCCN-Bold", fontSize=8.2, fillColor=BLACK, textAnchor="middle"))
        d.add(String(x + box_w / 2, y + 19, detail, fontName="SWTCCN", fontSize=7.1, fillColor=BLACK, textAnchor="middle"))
        if index < len(steps) - 1:
            start_x = x + box_w
            end_x = start_x + gap
            line_y = y + box_h / 2
            d.add(Line(start_x, line_y, end_x, line_y, strokeColor=accent, strokeWidth=1.2))
            d.add(Polygon([end_x, line_y, end_x - 5, line_y + 3, end_x - 5, line_y - 3], fillColor=accent, strokeColor=accent))
    return d


def score_breakdown_diagram() -> Drawing:
    width = 17.2 * cm
    height = 7.0 * cm
    d = Drawing(width, height)
    d.add(String(8, height - 18, "官网分项原始计分", fontName="SWTCCN-Bold", fontSize=10, fillColor=BLACK))
    values = [
        ("basic", 408.0, RED),
        ("BusyBox", 208.0, BLUE),
        ("Lua", 36.0, TEAL),
        ("libcbench", 86.84477333493804, colors.HexColor("#D08C32")),
        ("libctest", 217.0, colors.HexColor("#7B61A8")),
        ("LTP", 314.0, colors.HexColor("#4C8B63")),
    ]
    max_value = max(value for _, value, _ in values)
    left = 75
    max_bar = width - left - 70
    top = height - 48
    row_h = 25
    for index, (name, value, color) in enumerate(values):
        y = top - index * row_h
        d.add(String(left - 8, y + 3, name, fontName="SWTCCN", fontSize=8.2, fillColor=BLACK, textAnchor="end"))
        bar_w = max_bar * value / max_value
        d.add(Rect(left, y - 4, max_bar, 12, fillColor=PALE_GRAY, strokeColor=None))
        d.add(Rect(left, y - 4, bar_w, 12, fillColor=color, strokeColor=None))
        label = f"{value:.6f}" if value % 1 else f"{int(value)}"
        d.add(String(left + bar_w + 7, y + 1, label, fontName="SWTCCN", fontSize=7.5, fillColor=BLACK))
    d.add(String(8, 11, "注：图中为官网各测试组原始分项；平台总分按官方计分规则显示为 1009.8903911255481。", fontName="SWTCCN", fontSize=7.3, fillColor=BLACK))
    return d


def image_for_width(path: Path, width: float, max_height: float | None = None) -> Image:
    with PILImage.open(path) as image:
        px_w, px_h = image.size
    height = width * px_h / px_w
    if max_height is not None and height > max_height:
        height = max_height
        width = height * px_w / px_h
    return Image(str(path), width=width, height=height)


def page_header_footer(canvas, doc) -> None:
    if doc.page == 1:
        return
    canvas.saveState()
    canvas.setStrokeColor(LIGHT_LINE)
    canvas.setLineWidth(0.5)
    canvas.line(MARGIN_X, PAGE_HEIGHT - 1.18 * cm, PAGE_WIDTH - MARGIN_X, PAGE_HEIGHT - 1.18 * cm)
    canvas.setFont("SWTCCN", 8.3)
    canvas.setFillColor(BLACK)
    canvas.drawString(MARGIN_X, PAGE_HEIGHT - 0.95 * cm, "SWTC 初赛作品文档")
    canvas.drawRightString(PAGE_WIDTH - MARGIN_X, PAGE_HEIGHT - 0.95 * cm, "上海电力大学 · sudo_win_the_cscc")
    canvas.setStrokeColor(LIGHT_LINE)
    canvas.line(MARGIN_X, 1.05 * cm, PAGE_WIDTH - MARGIN_X, 1.05 * cm)
    canvas.drawCentredString(PAGE_WIDTH / 2, 0.65 * cm, f"第 {doc.page} 页")
    canvas.restoreState()


def add_page_number(canvas, doc) -> None:
    page_header_footer(canvas, doc)


class SWTCDocTemplate(BaseDocTemplate):
    """Document template with PDF outline and generated table of contents."""

    def beforeDocument(self) -> None:
        super().beforeDocument()
        self._heading_index = 0

    def afterFlowable(self, flowable) -> None:
        if not isinstance(flowable, Paragraph):
            return
        if flowable.style.name not in {"H1", "H2"}:
            return
        level = 0 if flowable.style.name == "H1" else 1
        text = flowable.getPlainText()
        key = f"heading-{self._heading_index}"
        self._heading_index += 1
        self.canv.bookmarkPage(key)
        self.canv.addOutlineEntry(text, key, level=level, closed=False)
        self.notify("TOCEntry", (level, text, self.page, key))


def build_story(styles: dict[str, ParagraphStyle]) -> list[object]:
    logo_path = ASSET_DIR / "shanghai-university-of-electric-power-logo.png"
    evaluation_path = ASSET_DIR / "official-evaluation-2026-06-30.png"
    architecture_path = ASSET_DIR / "swtc-system-architecture.png"

    story: list[object] = []

    # Cover
    story.append(Spacer(1, 0.8 * cm))
    story.append(image_for_width(logo_path, 15.5 * cm, 3.7 * cm))
    story.append(Spacer(1, 1.35 * cm))
    story.append(p("2026 年全国大学生计算机系统能力大赛<br/>操作系统设计赛（内核实现赛道）", styles["cover_comp"]))
    story.append(Spacer(1, 1.1 * cm))
    story.append(p("SWTC 初赛作品文档", styles["cover_title"]))
    story.append(p("面向官方评测的 RISC-V64 / LoongArch64 双架构操作系统内核", styles["cover_subtitle"]))
    story.append(Spacer(1, 2.0 * cm))
    cover_data = [
        [p("参赛学校", styles["table_center"]), p("上海电力大学", styles["table_center"])],
        [p("参赛队伍", styles["table_center"]), p("sudo_win_the_cscc", styles["table_center"])],
        [p("作品名称", styles["table_center"]), p("SWTC", styles["table_center"])],
        [p("指导教师", styles["table_center"]), p("徐曼", styles["table_center"])],
        [p("队伍成员", styles["table_center"]), p("余子豪、王溢敏", styles["table_center"])],
        [p("文档日期", styles["table_center"]), p("2026 年 6 月 30 日", styles["table_center"])],
    ]
    cover_table = Table(cover_data, colWidths=[4.2 * cm, 9.5 * cm], hAlign="CENTER")
    cover_table.setStyle(
        TableStyle(
            [
                ("FONTNAME", (0, 0), (-1, -1), "SWTCCN"),
                ("BACKGROUND", (0, 0), (0, -1), PALE_RED),
                ("BACKGROUND", (1, 0), (1, -1), PALE_GRAY),
                ("TEXTCOLOR", (0, 0), (0, -1), BLACK),
                ("FONTNAME", (0, 0), (0, -1), "SWTCCN-Bold"),
                ("GRID", (0, 0), (-1, -1), 0.5, LIGHT_LINE),
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("TOPPADDING", (0, 0), (-1, -1), 8),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 8),
            ]
        )
    )
    story.append(cover_table)
    story.append(Spacer(1, 1.2 * cm))
    story.append(p("初赛提交版", styles["cover_subtitle"]))
    story.append(PageBreak())

    # Front matter: abstract
    story.append(p("摘 要", styles["front_title"]))
    story.append(
        p(
            "SWTC 是一款面向 2026 全国大学生计算机系统能力大赛操作系统设计赛官方评测环境开发的双架构内核。项目在同一仓库维护 RISC-V64 与 LoongArch64 两条实现主线，通过统一的根构建入口生成 kernel-rv 与 kernel-la，并直接执行官方无分区 EXT4 测试盘中的预编译 ELF、动态运行时和测试脚本。",
            styles["body"],
        )
    )
    story.append(
        p(
            "初赛阶段，SWTC 重点建设从离线源码构建、架构正确性检查、块设备发现、脚本解析、资源暂存、ELF 装载、进程执行、Linux syscall 兼容、结果输出到主动关机的完整评测闭环。RISC-V 主线以 Titanix 开源架构为基础完成比赛适配；LoongArch 主线基于 StarryX/ArceOS 组件体系完成真实架构支持。本队新增工作集中于双架构统一提交、官方评测入口、Linux ABI 兼容、故障隔离、离线依赖和验证门禁。",
            styles["body"],
        )
    )
    story.append(
        p(
            "截至 2026 年 6 月 30 日，SWTC 当前最高且最新可见官网成绩为 1009.8903911255481。四列 basic 均为 102，BusyBox 与 Lua 四列均已计分，三列 libcbench 已计分，musl-rv libctest 为 217、LTP 为 314。该结果证明双架构构建、主要应用入口及 RISC-V functional 路径已经形成有效计分能力。",
            styles["body"],
        )
    )
    story.append(p("关键词：Rust；操作系统内核；RISC-V64；LoongArch64；Linux ABI；EXT4；官方评测", styles["quote"]))
    story.append(Spacer(1, 0.3 * cm))
    story.append(image_for_width(evaluation_path, 13.8 * cm, 10.2 * cm))
    story.append(p("图 0-1  sudo_win_the_cscc 官网评测列表记录", styles["caption"]))
    story.append(PageBreak())

    # Front matter: module completion
    story.append(p("模块完成情况", styles["front_title"]))
    module_rows = [
        [table_cell("模块", styles, True), table_cell("完成情况", styles, True), table_cell("验证证据", styles, True)],
        [table_cell("统一构建", styles, True), table_cell("根目录 make all 生成双架构 ELF；工具链、离线依赖与源树门禁已接入", styles), table_cell("干净导出构建通过", styles)],
        [table_cell("RISC-V64", styles, True), table_cell("启动、trap、进程、内存、文件系统、网络、信号及 Linux syscall 基线已接入", styles), table_cell("官方多个测试组线上计分", styles)],
        [table_cell("LoongArch64", styles, True), table_cell("真实 kernel-la、平台配置、用户空间、VFS、EXT4 与官方脚本入口已接入", styles), table_cell("basic/BusyBox/Lua/libcbench 线上计分", styles)],
        [table_cell("测试盘入口", styles, True), table_cell("识别无分区 EXT4，支持固定路径、目录遍历、脚本和 ELF 读取", styles), table_cell("glibc/musl 双目录回归", styles)],
        [table_cell("ELF 与运行时", styles, True), table_cell("静态/动态 ELF、PT_INTERP、argv/envp/auxv 与动态解释器暂存", styles), table_cell("basic 与 libctest 动态路径计分", styles)],
        [table_cell("应用测试", styles, True), table_cell("basic、BusyBox、Lua、libcbench、libctest、LTP 已形成分组执行路径", styles), table_cell("官网总分 1009.8903911255481", styles)],
        [table_cell("故障隔离", styles, True), table_cell("组私有工作目录、逐项 timeout、START/END 收尾与主动关机", styles), table_cell("长队列失败后仍保留前序得分", styles)],
        [table_cell("验证体系", styles, True), table_cell("源码完整性、vendor checksum、ELF 架构、无盘 QEMU 与官方镜像回归", styles), table_cell("提交前多层门禁", styles)],
        [table_cell("待完善项", styles, True), table_cell("LoongArch libctest/LTP、性能测试与更完整的跨架构行为一致性", styles), table_cell("后续按 case 扩展", styles)],
    ]
    story.append(make_table(module_rows, [3.0 * cm, 9.2 * cm, 4.6 * cm], compact=True))
    story.append(Spacer(1, 0.5 * cm))
    story.append(
        p(
            "完成情况采用“源码实现 - 本地回归 - 线上计分”三层证据描述。对于尚未获得线上证据的能力，文档按当前验证状态如实标注；基础开源项目能力与本队比赛适配工作在后续章节分别说明。",
            styles["quote"],
        )
    )
    story.append(PageBreak())

    # Front matter: generated table of contents
    story.append(p("目 录", styles["front_title"]))
    toc = TableOfContents()
    toc.levelStyles = [styles["toc_level0"], styles["toc_level1"]]
    toc.dotsMinLevel = 0
    story.append(toc)
    story.append(PageBreak())

    # 1. Basic information
    story.append(p("1. 作品概况", styles["h1"]))
    story.append(p("1.1 作品基本信息", styles["h2"]))
    basic_rows = [
        [table_cell("项目", styles, True), table_cell("内容", styles, True)],
        [table_cell("队伍名称", styles), table_cell("sudo_win_the_cscc", styles)],
        [table_cell("操作系统名称", styles), table_cell("SWTC", styles)],
        [table_cell("支持架构", styles), table_cell("RISC-V64、LoongArch64", styles)],
        [table_cell("主要语言", styles), table_cell("Rust、Makefile、少量汇编与 Shell", styles)],
        [table_cell("官方产物", styles), table_cell("kernel-rv、kernel-la", styles)],
        [table_cell("构建方式", styles), table_cell("仓库根目录执行 make all，离线完成双架构构建", styles)],
        [table_cell("测试方向", styles), table_cell("basic、BusyBox、Lua、libcbench、libctest、LTP", styles)],
    ]
    story.append(make_table(basic_rows, [4.0 * cm, 12.8 * cm]))
    story.append(Spacer(1, 0.35 * cm))
    story.append(p("1.2 项目人员", styles["h2"]))
    people_rows = [
        [table_cell("参赛身份", styles, True), table_cell("姓名", styles, True), table_cell("学校与专业", styles, True)],
        [table_cell("指导教师", styles, True), table_cell("徐曼", styles, True), table_cell("上海电力大学", styles)],
        [table_cell("队长、队员", styles, True), table_cell("余子豪", styles, True), table_cell("上海电力大学，计算机科学与技术（卓越计划）", styles)],
        [table_cell("队员", styles, True), table_cell("王溢敏", styles, True), table_cell("上海电力大学，计算机科学与技术（卓越计划）", styles)],
    ]
    story.append(make_table(people_rows, [3.1 * cm, 3.0 * cm, 10.7 * cm]))
    story.append(Spacer(1, 0.35 * cm))
    story.append(p("1.3 作品概述", styles["h2"]))
    story.append(
        p(
            "SWTC 是面向 2026 操作系统设计赛官方评测环境开发的双架构操作系统内核。项目在同一仓库、同一提交入口下维护 RISC-V64 与 LoongArch64 两条实现主线，目标是稳定生成官方要求的两个内核 ELF，并执行测试盘中的 Linux 用户态程序。",
            styles["body"],
        )
    )
    story.append(
        p(
            "项目围绕官方无分区 EXT4 测试盘、预编译 ELF 和测试脚本构建端到端运行链路：内核启动后识别块设备，读取脚本与资源，组织测试队列，装载用户程序，提供最小 Linux ABI，按官方格式输出结果并主动关机。当前最高且最新可见官网成绩为 1009.8903911255481。",
            styles["body"],
        )
    )
    story.append(p("项目以“可构建、可启动、可执行、可计分、可复现”为初赛工程目标。", styles["quote"]))
    story.append(PageBreak())

    # 2. Architecture
    story.append(p("2. 系统总体架构", styles["h1"]))
    story.append(
        p(
            "SWTC 采用统一提交入口与双架构实现并行的工程组织方式。根构建负责选择各自工具链、恢复离线依赖配置、检查源树完整性并生成官方产物；运行时则统一面向官方测试盘和日志协议。",
            styles["body"],
        )
    )
    story.append(Spacer(1, 0.2 * cm))
    story.append(image_for_width(architecture_path, 17.2 * cm, 9.5 * cm))
    story.append(p("图 1  SWTC 统一构建、官方运行环境与内核运行路径", styles["caption"]))
    story.append(p("2.1 双架构组织", styles["h2"]))
    story.append(
        bullets(
            [
                "RISC-V64 主线位于 SWTC/，以 Titanix 往届开源架构为基础，重点完成官方镜像入口、Linux ABI、EXT4 读取、脚本解析和测试执行适配。",
                "LoongArch64 主线位于 SWTC-la/，基于 StarryX/ArceOS 组件化内核进行比赛适配，重点完成工具链、真实 kernel-la、测试盘和用户态应用运行。",
                "两条主线共享官方提交约束、测试组命名、日志格式、回归标准与文档记录方式。",
            ],
            styles,
        )
    )
    story.append(p("2.2 核心数据流", styles["h2"]))
    flow_rows = [
        [table_cell("阶段", styles, True), table_cell("输入", styles, True), table_cell("核心处理", styles, True), table_cell("输出", styles, True)],
        [table_cell("构建", styles, True), table_cell("GitLab 源码", styles), table_cell("双工具链离线构建、源树门禁", styles), table_cell("kernel-rv / kernel-la", styles)],
        [table_cell("发现", styles, True), table_cell("virtio-blk", styles), table_cell("识别无分区 EXT4 与固定脚本路径", styles), table_cell("测试组与资源索引", styles)],
        [table_cell("执行", styles, True), table_cell("脚本、ELF、运行时", styles), table_cell("staging、ELF 装载、进程与 syscall", styles), table_cell("测试结果", styles)],
        [table_cell("收尾", styles, True), table_cell("case 状态", styles), table_cell("官方 START/END 格式、超时隔离", styles), table_cell("计分日志与主动关机", styles)],
    ]
    story.append(make_table(flow_rows, [2.2 * cm, 3.3 * cm, 7.1 * cm, 4.2 * cm], compact=True))
    story.append(PageBreak())

    story.append(p("2.3 代码组织与模块边界", styles["h2"]))
    story.append(
        p(
            "SWTC 保留两套能够独立构建的内核主体，并在仓库根目录建立统一交付层。这样的组织方式允许两条主线根据架构差异选择不同内核基础，同时用相同的提交规范、测试命名和输出协议约束最终行为。",
            styles["body"],
        )
    )
    layout_rows = [
        [table_cell("路径", styles, True), table_cell("职责", styles, True), table_cell("主要内容", styles, True)],
        [table_cell("Makefile / GNUmakefile", styles), table_cell("统一交付层", styles), table_cell("工具链检查、隐藏配置恢复、双架构构建、产物门禁", styles)],
        [table_cell("SWTC/kernel", styles), table_cell("RISC-V64 内核", styles), table_cell("启动、trap、进程、内存、文件系统、网络、信号、syscall 与 oscomp", styles)],
        [table_cell("SWTC/user", styles), table_cell("RISC-V 用户态", styles), table_cell("init/runtestcase、测试队列消费与官方输出协议", styles)],
        [table_cell("SWTC-la/src", styles), table_cell("LoongArch 入口", styles), table_cell("平台启动、用户应用执行、syscall 接口与 init.sh", styles)],
        [table_cell("SWTC-la/xcore", styles), table_cell("LoongArch 核心服务", styles), table_cell("VFS、fd、syscall、调度与运行时基础", styles)],
        [table_cell("SWTC-la/xmodules", styles), table_cell("组件化能力", styles), table_cell("xprocess、xsignal、xuspace、xvma、xcache、xapi", styles)],
        [table_cell("docs / tools", styles), table_cell("验证与文档", styles), table_cell("进度证据、架构图、PDF 生成与 vendor 校验脚本", styles)],
    ]
    story.append(make_table(layout_rows, [4.0 * cm, 3.2 * cm, 9.6 * cm]))
    story.append(Spacer(1, 0.45 * cm))
    story.append(p("2.4 双架构职责划分", styles["h2"]))
    story.append(
        bullets(
            [
                "统一部分：官方构建入口、产物名称、测试盘约定、glibc/musl 测试组、START/END 输出和主动退出。",
                "RISC-V 特有部分：高地址内核二进制 wrapper、oscomp 内核态测试资产 staging、runtestcase 队列执行器。",
                "LoongArch 特有部分：DMW/高地址映射、LoongArch 平台配置、ArceOS/StarryX 组件组合和 BusyBox init.sh 调度。",
                "一致性目标：相同测试组在两条主线上使用相同资源、超时原则和结果格式，降低架构间行为偏差。",
            ],
            styles,
        )
    )
    story.append(PageBreak())

    # 3. Work content
    story.append(p("3. 主要工作内容", styles["h1"]))
    sections = [
        (
            "3.1 官方构建入口与离线依赖",
            [
                "根目录保留 Makefile 与 GNUmakefile，执行 make all 统一生成两个官方 ELF。",
                "RISC-V 与 LoongArch 使用各自固定 nightly 工具链，避免跨架构工具链和 target 配置相互污染。",
                "针对官方提交过滤隐藏文件的约束，构建时从非隐藏目录恢复 Cargo 配置与 checksum，并对 vendor 完整性进行检查。",
                "LoongArch 构建默认拒绝使用 RISC-V 产物充当 kernel-la，使架构错误在编译阶段直接暴露。",
            ],
        ),
        (
            "3.2 官方测试盘与脚本执行",
            [
                "通过 virtio-blk 访问无分区 EXT4 测试盘，支持多级目录下普通文件与脚本的只读访问。",
                "按 glibc、musl 和根目录路径定位测试组，解析脚本并形成可控的串行执行队列。",
                "将 ELF、动态解释器、脚本和数据文件暂存到组私有目录，降低跨组资源污染。",
                "对高风险测试采用逐 case 超时和组级 START/END 收尾，防止单项卡死导致整轮评测失效。",
            ],
        ),
        (
            "3.3 Linux ABI 与进程执行",
            [
                "支持 ELF PT_LOAD/PT_INTERP 处理，构造 argc、argv、envp 与 auxv，并执行动态或静态用户程序。",
                "逐步实现文件描述符、路径、内存、进程、时间、信号与 futex 等 Linux syscall 兼容语义。",
                "建立 fork/clone、execve、wait4 与超时控制的最小执行链，满足测试程序父子进程和线程场景。",
                "根据线上日志和本地镜像结果定位缺失 ABI，按可验证的小批次扩展 LTP 覆盖。",
            ],
        ),
        (
            "3.4 LoongArch64 实现与适配",
            [
                "完成 LoongArch64 内核启动、离线 vendor、配置生成器与 GCC 13.2 musl 工具链适配。",
                "真实 kernel-la 为 LoongArch ELF，入口地址 0x80000000；本地已验证 basic 与 BusyBox 测试组可运行并主动退出。",
                "将易受官方源包过滤影响的核心依赖迁移到非隐藏目录，并在根构建中增加源树完整性门禁。",
                "对测试顺序与工作目录进行隔离，避免 BusyBox 递归扫描大型 LTP 目录而中断后续测试。",
            ],
        ),
    ]
    for heading, items in sections:
        story.append(KeepTogether([p(heading, styles["h2"]), bullets(items, styles)]))
    story.append(PageBreak())

    story.append(p("3.5 双工具链构建系统", styles["h2"]))
    story.append(
        p(
            "官方评测只执行仓库根目录的 make all，因此根构建既是交付入口，也是第一道质量门禁。SWTC 不让两个架构共享一个模糊的默认工具链，而是显式固定 RISC-V nightly-2025-02-01 与 LoongArch nightly-2025-05-20，并分别检查 target、rust-lld、rust-objcopy、CMake 和 loongarch64-linux-musl-gcc。",
            styles["body"],
        )
    )
    story.append(
        pipeline_diagram(
            "构建与交付流水线",
            [
                ("源码检查", "关键路径完整"),
                ("配置恢复", "Cargo/checksum"),
                ("RISC-V", "wrapper ELF"),
                ("LoongArch", "真实架构 ELF"),
                ("产物门禁", "kernel-rv/la"),
            ],
        )
    )
    build_rows = [
        [table_cell("构建项", styles, True), table_cell("RISC-V64", styles, True), table_cell("LoongArch64", styles, True)],
        [table_cell("Rust 工具链", styles), table_cell("nightly-2025-02-01", styles), table_cell("nightly-2025-05-20", styles)],
        [table_cell("目标三元组", styles), table_cell("riscv64gc-unknown-none-elf", styles), table_cell("loongarch64-unknown-none", styles)],
        [table_cell("最终入口", styles), table_cell("0x80200000", styles), table_cell("0x80000000", styles)],
        [table_cell("关键转换", styles), table_cell("raw binary + wrapper object + rust-lld", styles), table_cell("直接复制真实 LoongArch ELF", styles)],
        [table_cell("失败策略", styles), table_cell("缺 target/tool 立即失败", styles), table_cell("默认拒绝跨架构 placeholder", styles)],
    ]
    story.append(make_table(build_rows, [3.4 * cm, 6.7 * cm, 6.7 * cm], compact=True))
    story.append(code_block("""
all: submission-info sanitize-submit-workdir build
build: build-rv build-la
build-rv: prepare-rv -> user -> kernel -> wrapper ELF
build-la: prepare-la -> strict LoongArch build -> kernel-la
""", styles))
    story.append(
        p(
            "官方会过滤隐藏文件，因此 Cargo 配置和 vendor checksum 的可恢复副本存放在非隐藏路径。构建阶段再恢复 .cargo/config.toml 与 .cargo-checksum.json；该设计不要求评测机联网，也不依赖工作区中的历史缓存。",
            styles["quote"],
        )
    )
    story.append(PageBreak())

    story.append(p("3.6 官方测试资产发现与受控执行", styles["h2"]))
    story.append(
        p(
            "RISC-V 的 oscomp 模块直接从 virtio-blk 读取 EXT4 元数据，支持 extent tree、传统块映射、目录项和多级路径解析。测试组首先按固定路径发现 glibc/ 与 musl/ 下的脚本，再读取脚本、ELF、动态解释器和数据文件，并将其安装到 tmpfs 组私有目录。",
            styles["body"],
        )
    )
    story.append(
        pipeline_diagram(
            "官方测试执行闭环",
            [
                ("发现", "EXT4 固定路径"),
                ("解析", "脚本/ELF/interp"),
                ("暂存", "tmpfs 私有目录"),
                ("执行", "fork/exec/wait"),
                ("收尾", "日志与关机"),
            ],
            accent=TEAL,
        )
    )
    queue_rows = [
        [table_cell("记录", styles, True), table_cell("含义", styles, True), table_cell("执行行为", styles, True)],
        [table_cell("G", styles, True), table_cell("进入测试组", styles), table_cell("切换组私有工作目录并输出 START", styles)],
        [table_cell("X", styles, True), table_cell("基础可执行文件", styles), table_cell("执行单个 ELF 并等待退出", styles)],
        [table_cell("A", styles, True), table_cell("argv + timeout", styles), table_cell("携带参数执行 BusyBox、Lua、bench 等", styles)],
        [table_cell("C", styles, True), table_cell("libctest case", styles), table_cell("按 case 输出 RUN/Pass/FAIL", styles)],
        [table_cell("L", styles, True), table_cell("LTP case", styles), table_cell("逐项限时并保留 case 收尾状态", styles)],
        [table_cell("E", styles, True), table_cell("结束标记", styles), table_cell("无论前序成败都输出组 END", styles)],
    ]
    story.append(make_table(queue_rows, [2.0 * cm, 4.2 * cm, 10.6 * cm], compact=True))
    story.append(p("执行队列采用 NUL 分隔记录，避免脚本中的空格和换行破坏边界。libctest、LTP 与 lmbench 分别设置独立 timeout；超时只终止当前项，不应吞掉整个测试组。", styles["quote"]))
    story.append(PageBreak())

    story.append(p("3.7 RISC-V64 内核主线", styles["h2"]))
    story.append(
        p(
            "RISC-V64 主线位于 SWTC/kernel。其基础架构来自 Titanix GPLv3 开源项目，本队在保留许可证和来源说明的前提下，将工作重点放在 2026 官方测试资产入口、离线构建、动态运行时、结果协议及 Linux ABI 差异修复。",
            styles["body"],
        )
    )
    rv_rows = [
        [table_cell("子系统", styles, True), table_cell("当前结构", styles, True), table_cell("比赛相关工作", styles, True)],
        [table_cell("启动与 trap", styles), table_cell("entry.S、trap.S、用户/内核 trap 分离", styles), table_cell("官方 QEMU 入口、主动退出与异常诊断", styles)],
        [table_cell("进程", styles), table_cell("Process/Thread、资源对象、调度与 wait", styles), table_cell("fork/clone/execve/wait4 与测试超时链路", styles)],
        [table_cell("内存", styles), table_cell("页表、VMA、COW、共享内存与用户指针检查", styles), table_cell("ELF PT_LOAD/PT_INTERP、brk/mmap 与边界语义", styles)],
        [table_cell("文件系统", styles), table_cell("VFS、fd table、FAT32/tmpfs/procfs/devfs、页缓存", styles), table_cell("EXT4 测试盘只读导入与组资源 staging", styles)],
        [table_cell("同步/时间", styles), table_cell("futex、mutex、poll、timeout 与定时任务", styles), table_cell("libcbench/LTP 所需等待和时间语义", styles)],
        [table_cell("网络/信号", styles), table_cell("TCP/UDP/Unix socket、标准信号路径", styles), table_cell("保留基础能力，初赛优先 functional 计分", styles)],
        [table_cell("oscomp", styles), table_cell("约 2200 行评测适配模块", styles), table_cell("EXT4、脚本、ELF、队列、运行时与测试组编排", styles)],
    ]
    story.append(make_table(rv_rows, [2.7 * cm, 6.5 * cm, 7.6 * cm], compact=True))
    story.append(Spacer(1, 0.35 * cm))
    story.append(p("RISC-V 主线的关键价值在于将已有内核能力转化为官方可观察、可计分的行为。实际适配中，很多问题并非单纯缺少一个 syscall 编号，而是 argv、结构体布局、errno、阻塞/超时、动态解释器路径和输出边界等 Linux ABI 语义细节存在偏差。", styles["quote"]))
    story.append(PageBreak())

    story.append(p("3.8 LoongArch64 内核主线", styles["h2"]))
    story.append(
        p(
            "LoongArch64 主线位于 SWTC-la，以 StarryX/ArceOS 组件化体系为基础。入口程序初始化进程、挂载 VFS、建立标准输入输出，并以 /musl/busybox sh -c 执行内嵌 init.sh。xcore 提供核心服务，xprocess、xsignal、xuspace、xvma、xcache 等模块分别承载进程、信号、用户空间、虚拟内存和缓存能力。",
            styles["body"],
        )
    )
    la_rows = [
        [table_cell("层次", styles, True), table_cell("实现", styles, True), table_cell("比赛适配", styles, True)],
        [table_cell("平台", styles), table_cell("loongarch64-qemu-virt，物理入口 0x80000000", styles), table_cell("UART、GED、PCI、virtio-blk 与时钟配置", styles)],
        [table_cell("地址空间", styles), table_cell("DMW 启动窗口 + 高地址内核映射", styles), table_cell("分页开启前后保持启动代码连续执行", styles)],
        [table_cell("运行时", styles), table_cell("ArceOS axruntime/axtask/axmm/axfs-ng", styles), table_cell("真实 kernel-la 与用户态 ELF 执行", styles)],
        [table_cell("文件系统", styles), table_cell("VFS + lwext4_rust + fd/stdio", styles), table_cell("直接挂载官方测试盘并访问 glibc/musl 目录", styles)],
        [table_cell("测试编排", styles), table_cell("BusyBox init.sh", styles), table_cell("动态库链接、工作目录沙箱、逐项 timeout", styles)],
        [table_cell("离线依赖", styles), table_cell("vendor + 本地 xapi/xcore/xmodules", styles), table_cell("规避官方源包遗漏与 crates.io 网络依赖", styles)],
    ]
    story.append(make_table(la_rows, [2.6 * cm, 6.2 * cm, 8.0 * cm], compact=True))
    story.append(code_block("""
fn main() {
    Process::new_init(current_task_id).build();
    vfs::init_root();
    fd::init_stdio();
    run_user_app(["/musl/busybox", "sh", "-c", init_script]);
    terminate();
}
""", styles))
    story.append(p("LoongArch 线上已获得 basic、BusyBox、Lua 与 musl libcbench 分数；libctest/LTP 尚未形成分数，是后续最明确的增长方向。", styles["quote"]))
    story.append(PageBreak())

    # 4. Completion
    story.append(p("4. 完成度与验证状态", styles["h1"]))
    story.append(
        p(
            "以下状态以已完成的本地回归和可见官网结果为依据。线上得分受评测源包、排队与运行环境影响，因此文档将“实现完成”“本地验证”和“线上计分”分开描述。",
            styles["body"],
        )
    )
    completion_rows = [
        [table_cell("测试/模块", styles, True), table_cell("RISC-V64", styles, True), table_cell("LoongArch64", styles, True), table_cell("当前说明", styles, True)],
        [table_cell("根构建", styles, True), table_cell("已通过", styles, True), table_cell("已通过", styles, True), table_cell("干净导出可生成两个架构正确的 ELF", styles)],
        [table_cell("basic", styles, True), table_cell("线上稳定得分", styles), table_cell("本地 64/64，曾线上得分", styles), table_cell("双 libc 路径已接入", styles)],
        [table_cell("BusyBox", styles, True), table_cell("线上得分", styles), table_cell("本地 55/55，曾线上得分", styles), table_cell("LA 已隔离工作目录", styles)],
        [table_cell("Lua", styles, True), table_cell("线上得分", styles), table_cell("已接入", styles), table_cell("依赖 Lua 与脚本资源 staging", styles)],
        [table_cell("libcbench", styles, True), table_cell("线上得分", styles), table_cell("部分接入", styles), table_cell("高风险 glibc 项已后置隔离", styles)],
        [table_cell("libctest", styles, True), table_cell("musl 线上得分", styles), table_cell("已接入", styles), table_cell("LA 逐 case 继续验证", styles)],
        [table_cell("LTP", styles, True), table_cell("稳定子集线上得分", styles), table_cell("部分接入", styles), table_cell("按 syscall 能力分批扩展", styles)],
        [table_cell("EXT4/tmpfs", styles, True), table_cell("已接入", styles), table_cell("已接入", styles), table_cell("只读测试盘 + 运行资源暂存", styles)],
        [table_cell("进程模型", styles, True), table_cell("最小 Linux ABI", styles), table_cell("组件化内核适配", styles), table_cell("尚非完整 Linux 隔离模型", styles)],
    ]
    story.append(make_table(completion_rows, [2.6 * cm, 3.9 * cm, 4.2 * cm, 6.1 * cm], compact=True))
    story.append(Spacer(1, 0.45 * cm))
    story.append(p("4.1 已建立的回归门禁", styles["h2"]))
    story.append(
        bullets(
            [
                "干净导出构建：确保源码包不依赖本机未提交文件或缓存。",
                "ELF 架构检查：kernel-rv 必须为 RISC-V ELF，kernel-la 必须为 LoongArch ELF。",
                "无盘 QEMU 回归：没有测试盘时也能启动、输出并主动退出。",
                "官方镜像回归：检查测试组 START/END、case 输出、超时行为与最终关机。",
                "线上结果回读：以官网分项和串口日志更新进度，避免凭本地现象推断线上结论。",
            ],
            styles,
        )
    )
    story.append(p("当前最高且最新可见官网成绩：1009.8903911255481（提交时间 2026-06-30 09:17:33，Accepted）。", styles["quote"]))
    story.append(PageBreak())

    story.append(p("4.2 线上分项结果", styles["h2"]))
    story.append(
        p(
            "官网结果显示四列 basic 全部为 102，BusyBox 四列为 54、50、54、50，Lua 四列均为 9。libcbench 已在 glibc-rv、musl-la 与 musl-rv 三列计分；musl-rv 的 libctest 与 LTP 分别为 217 和 314。",
            styles["body"],
        )
    )
    story.append(score_breakdown_diagram())
    score_rows = [
        [table_cell("测试组", styles, True), table_cell("glibc-la", styles, True), table_cell("glibc-rv", styles, True), table_cell("musl-la", styles, True), table_cell("musl-rv", styles, True), table_cell("分项", styles, True)],
        [table_cell("basic", styles), table_cell("102", styles, True), table_cell("102", styles, True), table_cell("102", styles, True), table_cell("102", styles, True), table_cell("408", styles, True)],
        [table_cell("BusyBox", styles), table_cell("54", styles, True), table_cell("50", styles, True), table_cell("54", styles, True), table_cell("50", styles, True), table_cell("208", styles, True)],
        [table_cell("Lua", styles), table_cell("9", styles, True), table_cell("9", styles, True), table_cell("9", styles, True), table_cell("9", styles, True), table_cell("36", styles, True)],
        [table_cell("libcbench", styles), table_cell("0", styles, True), table_cell("30.3442", styles, True), table_cell("29.3344", styles, True), table_cell("27.1661", styles, True), table_cell("86.8448", styles, True)],
        [table_cell("libctest", styles), table_cell("-", styles, True), table_cell("-", styles, True), table_cell("0", styles, True), table_cell("217", styles, True), table_cell("217", styles, True)],
        [table_cell("LTP", styles), table_cell("0", styles, True), table_cell("0", styles, True), table_cell("0", styles, True), table_cell("314", styles, True), table_cell("314", styles, True)],
    ]
    story.append(make_table(score_rows, [2.4 * cm, 2.7 * cm, 2.7 * cm, 2.7 * cm, 2.7 * cm, 3.6 * cm], compact=True))
    story.append(p("表内数值按官网得分详情原样记录；官网最终总分显示为 1009.8903911255481。", styles["caption"]))
    story.append(PageBreak())

    story.append(p("4.3 分层验证矩阵", styles["h2"]))
    story.append(
        p(
            "排队时间可能达到数小时，因此每次提交前需要尽量在本地消除可以确定的问题。SWTC 将验证分为源码、构建、产物、启动、功能、协议和线上七层；后一层不替代前一层。",
            styles["body"],
        )
    )
    verify_rows = [
        [table_cell("层级", styles, True), table_cell("检查内容", styles, True), table_cell("失败时含义", styles, True), table_cell("提交门禁", styles, True)],
        [table_cell("源码", styles, True), table_cell("关键目录、Cargo.toml、vendor checksum", styles), table_cell("官方导出后可能缺文件", styles), table_cell("必须通过", styles, True)],
        [table_cell("构建", styles, True), table_cell("离线 make all，不访问网络", styles), table_cell("工具链或隐藏配置依赖", styles), table_cell("必须通过", styles, True)],
        [table_cell("产物", styles, True), table_cell("ELF 架构与入口地址", styles), table_cell("kernel-la placeholder 或链接错误", styles), table_cell("必须通过", styles, True)],
        [table_cell("启动", styles, True), table_cell("官方 QEMU 命令、无盘主动退出", styles), table_cell("启动栈、地址、设备或关机错误", styles), table_cell("必须通过", styles, True)],
        [table_cell("功能", styles, True), table_cell("官方 EXT4 镜像与分组 case", styles), table_cell("脚本、ELF、syscall 或运行时语义错误", styles), table_cell("目标组通过", styles, True)],
        [table_cell("协议", styles, True), table_cell("START/END、RUN、Pass/FAIL 与 !TEST FINISH!", styles), table_cell("程序运行但计分器无法识别", styles), table_cell("必须通过", styles, True)],
        [table_cell("线上", styles, True), table_cell("总分、四列分项与串口日志", styles), table_cell("环境差异或未覆盖 ABI", styles), table_cell("形成新证据", styles, True)],
    ]
    story.append(make_table(verify_rows, [2.1 * cm, 5.7 * cm, 5.5 * cm, 3.5 * cm], compact=True))
    story.append(Spacer(1, 0.45 * cm))
    story.append(p("4.4 提交前验证原则", styles["h2"]))
    story.append(
        bullets(
            [
                "先验证无盘启动和主动退出，避免测试盘逻辑掩盖基础启动回归。",
                "再验证 basic、BusyBox、Lua 等稳定组，确认保分路径未被新功能破坏。",
                "新 LTP case 只有在 syscall 行为和错误路径明确时才加入 allowlist。",
                "高风险性能项后置运行，不能截断前面已经可计分的 functional 组。",
                "线上结果与对应提交建立关联，避免不同提交批次的页面结果混淆。",
            ],
            styles,
        )
    )
    story.append(PageBreak())

    # 5. Innovations
    story.append(p("5. 项目特点与创新点", styles["h1"]))
    innovation_rows = [
        [table_cell("方向", styles, True), table_cell("实现与价值", styles, True)],
        [table_cell("双架构统一提交", styles, True), table_cell("在同一仓库维护 RISC-V64 与 LoongArch64 两套内核、依赖和工具链，根构建统一产出官方要求的两个 ELF。", styles)],
        [table_cell("测试脚本受控执行", styles, True), table_cell("不依赖完整发行版环境，将官方脚本解析为内核可调度的命令队列，统一管理资源、顺序、超时和日志。", styles)],
        [table_cell("评测约束工程化", styles, True), table_cell("围绕隐藏文件过滤、离线构建、源树缺失和长排队等现实约束，设计非隐藏依赖、完整性门禁和干净导出验证。", styles)],
        [table_cell("日志驱动 ABI 演进", styles, True), table_cell("根据真实 ELF 行为和线上日志逐项补齐 Linux syscall 语义，以可回归的小批次扩展 functional 测试覆盖。", styles)],
        [table_cell("真实架构产物防线", styles, True), table_cell("禁止用 RISC-V ELF 冒充 kernel-la，构建阶段检查源树与 ELF 架构，使错误尽早暴露。", styles)],
        [table_cell("测试组故障隔离", styles, True), table_cell("通过组私有工作目录、逐 case 超时和高风险项后置，减少单个测试对整轮成绩的级联影响。", styles)],
    ]
    story.append(make_table(innovation_rows, [4.1 * cm, 12.7 * cm]))
    story.append(Spacer(1, 0.55 * cm))
    story.append(p("5.1 工程取舍", styles["h2"]))
    story.append(
        p(
            "初赛阶段优先保证官方入口、Linux ABI 和 functional 测试链路，不将网络、图形界面等展示性功能置于计分闭环之前。对于尚未稳定的 iozone、lmbench 和大规模 LTP，采用后置、分批和可回滚策略，以维护已验证基线的稳定性。",
            styles["body"],
        )
    )
    story.append(p("创新重点不在堆叠功能数量，而在双架构、真实测试资产和受限评测环境下建立稳定、可解释、可复现的内核运行链路。", styles["quote"]))
    story.append(PageBreak())

    story.append(p("5.2 关键工程决策", styles["h2"]))
    decision_rows = [
        [table_cell("决策", styles, True), table_cell("问题背景", styles, True), table_cell("采用方案", styles, True), table_cell("结果", styles, True)],
        [table_cell("统一根入口", styles), table_cell("评测固定执行 make all", styles), table_cell("根 Makefile/GNUmakefile 编排双架构", styles), table_cell("交付行为单一明确", styles)],
        [table_cell("离线依赖", styles), table_cell("评测环境不能访问 crates.io", styles), table_cell("vendor + 非隐藏 checksum/config 备份", styles), table_cell("干净导出可离线构建", styles)],
        [table_cell("严格 kernel-la", styles), table_cell("占位产物会在运行期静默丢分", styles), table_cell("默认构建失败而非复制 kernel-rv", styles), table_cell("架构错误提前暴露", styles)],
        [table_cell("固定路径优先", styles), table_cell("大镜像递归扫描成本高", styles), table_cell("优先 glibc/musl 固定脚本路径", styles), table_cell("缩短启动与发现时间", styles)],
        [table_cell("队列执行器", styles), table_cell("RISC-V 不依赖完整 shell", styles), table_cell("内核 staging + 用户态记录消费", styles), table_cell("参数、超时和边界可控", styles)],
        [table_cell("组级沙箱", styles), table_cell("BusyBox find/du 会递归大目录", styles), table_cell("复制必要资源到 /tmp 私有目录", styles), table_cell("避免扫描 LTP 树导致 trap", styles)],
        [table_cell("高风险后置", styles), table_cell("性能测试可能异常退出或超时", styles), table_cell("functional 优先，bench 后置限时", styles), table_cell("保留已经获得的分数", styles)],
    ]
    story.append(make_table(decision_rows, [2.7 * cm, 4.5 * cm, 5.6 * cm, 4.0 * cm], compact=True))
    story.append(Spacer(1, 0.4 * cm))
    story.append(p("5.2.1 为什么不是完全统一内核源码", styles["h2"]))
    story.append(
        p(
            "RISC-V 与 LoongArch 主线的开源基础不同，短期强行合并会引入大范围回归，且无法直接提高初赛得分。项目选择统一交付接口、测试协议和验证标准，而允许底层实现保持架构适配所需差异。这是一种面向截止时间与风险的工程取舍。",
            styles["body"],
        )
    )
    story.append(p("统一的对象是可观察行为与提交质量，而不是为了目录形式一致去重写已经能工作的内核。", styles["quote"]))
    story.append(PageBreak())

    story.append(p("5.3 问题定位方法与经验总结", styles["h2"]))
    story.append(
        p(
            "内核评测问题往往跨越构建环境、QEMU、文件系统、动态链接器、用户程序和计分器。SWTC 将问题定位拆解为连续证据链，每一步只验证一个假设，避免仅根据最终分数推断底层原因。",
            styles["body"],
        )
    )
    debug_rows = [
        [table_cell("现象", styles, True), table_cell("优先核查证据", styles, True), table_cell("判断方向", styles, True)],
        [table_cell("Compile Error", styles), table_cell("根 Makefile 标记、缺失路径、Cargo checksum", styles), table_cell("入口/源包/离线依赖问题，不是运行时 ABI", styles)],
        [table_cell("QEMU 无输出", styles), table_cell("ELF 架构、入口地址、串口和启动栈", styles), table_cell("产物或平台初始化错误", styles)],
        [table_cell("找到脚本但 0 分", styles), table_cell("START/END、真实 case 输出、计分格式", styles), table_cell("协议或执行器未真正运行 ELF", styles)],
        [table_cell("execve 返回 ENOENT", styles), table_cell("PT_INTERP 与动态库实际路径", styles), table_cell("动态解释器缺失，而非主 ELF 不存在", styles)],
        [table_cell("某组后全部归零", styles), table_cell("最后一条串口日志与 trap 类型", styles), table_cell("当前组截断后续，应先隔离/后置", styles)],
        [table_cell("本地通过线上失败", styles), table_cell("干净导出、隐藏文件过滤、工具链版本", styles), table_cell("本机缓存或未提交文件依赖", styles)],
    ]
    story.append(make_table(debug_rows, [3.2 * cm, 6.2 * cm, 7.4 * cm], compact=True))
    story.append(Spacer(1, 0.4 * cm))
    story.append(
        bullets(
            [
                "建立最小复现：先用单个 ELF、单个脚本和最小测试盘确认入口。",
                "同时记录正向证据和负向证据：不仅记录通过项，也记录暂缓打开某组测试的工程依据。",
                "分数回退时优先比较测试组执行顺序与最后有效日志，不以总分单点判断根因。",
                "将平台异常与代码问题分开：源包工作目录、重复压缩文件等平台现象需要独立记录。",
                "文档与代码同步：新增 syscall、构建门禁和测试组都应留下验证方式与已知限制。",
            ],
            styles,
        )
    )
    story.append(p("该定位方法的目标，是让每次线上提交都建立在明确证据与可回归验证之上。", styles["quote"]))
    story.append(PageBreak())

    # 6. Evaluation evidence
    story.append(p("6. 官网评测记录", styles["h1"]))
    story.append(
        p(
            "下图为 2026 年 6 月 30 日官网评测列表截图。红框行显示参赛队伍 sudo_win_the_cscc，学校为上海电力大学；多个分项列已获得稳定结果，证明 SWTC 已能进入官方评测链路并执行双架构相关测试。",
            styles["body"],
        )
    )
    story.append(Spacer(1, 0.25 * cm))
    story.append(image_for_width(evaluation_path, 17.0 * cm, 15.3 * cm))
    story.append(p("图 2  sudo_win_the_cscc 官网评测列表记录（2026-06-30）", styles["caption"]))
    story.append(
        p(
            "评测成绩是阶段性工程证据，不等同于完整 Linux 兼容度。项目后续仍将通过本地官方镜像回归和线上分项日志交叉验证各项能力。",
            styles["quote"],
        )
    )
    story.append(PageBreak())

    # 7. Build and roadmap
    story.append(p("7. 构建、验证与后续计划", styles["h1"]))
    story.append(p("7.1 构建方式", styles["h2"]))
    build_box = Table(
        [[p("<font color=\"#000000\">cd /mnt/e/WorkSpace/MinOS/os-kernel-contest<br/>make all</font>", styles["body_no_indent"])]],
        colWidths=[16.8 * cm],
    )
    build_box.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), colors.white),
                ("TEXTCOLOR", (0, 0), (-1, -1), BLACK),
                ("FONTNAME", (0, 0), (-1, -1), "SWTCCN"),
                ("LEFTPADDING", (0, 0), (-1, -1), 12),
                ("TOPPADDING", (0, 0), (-1, -1), 10),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 10),
                ("BOX", (0, 0), (-1, -1), 0.5, NAVY),
            ]
        )
    )
    story.append(build_box)
    story.append(Spacer(1, 0.3 * cm))
    story.append(
        bullets(
            [
                "kernel-rv：RISC-V executable ELF，入口地址 0x80200000。",
                "kernel-la：LoongArch executable ELF，入口地址 0x80000000。",
                "构建默认使用仓库内离线依赖，不要求评测机访问 crates.io。",
            ],
            styles,
        )
    )
    story.append(p("7.2 当前边界", styles["h2"]))
    boundary_rows = [
        [table_cell("边界", styles, True), table_cell("后续处理", styles, True)],
        [table_cell("进程与资源隔离仍是最小实现", styles), table_cell("继续完善 per-process fd、父子资源语义和独立内核栈", styles)],
        [table_cell("LoongArch libctest/LTP 覆盖不足", styles), table_cell("按 case 日志补齐 syscall 与运行时语义", styles)],
        [table_cell("高风险性能测试可能截断后续组", styles), table_cell("保持后置、限时和独立工作目录", styles)],
        [table_cell("双架构实现基础不同", styles), table_cell("统一测试协议、构建门禁和文档，逐步收敛公共行为", styles)],
    ]
    story.append(make_table(boundary_rows, [7.0 * cm, 9.8 * cm]))
    story.append(Spacer(1, 0.35 * cm))
    story.append(p("7.3 后续计划", styles["h2"]))
    roadmap_rows = [
        [table_cell("优先级", styles, True), table_cell("工作项", styles, True), table_cell("验收标准", styles, True)],
        [table_cell("P0", styles, True), table_cell("稳定双架构官方构建与启动", styles), table_cell("两个 ELF 架构正确，QEMU 可进入内核并主动退出", styles)],
        [table_cell("P1", styles, True), table_cell("巩固已验证高分路径", styles), table_cell("basic/BusyBox/Lua/libcbench 分项保持稳定", styles)],
        [table_cell("P2", styles, True), table_cell("扩展 LA libctest 与 LTP", styles), table_cell("逐 case 可解释，新增 syscall 有本地和线上证据", styles)],
        [table_cell("P3", styles, True), table_cell("完善进程与文件描述符模型", styles), table_cell("fork/exec/wait 与多程序资源隔离回归通过", styles)],
    ]
    story.append(make_table(roadmap_rows, [2.0 * cm, 6.2 * cm, 8.6 * cm]))
    story.append(PageBreak())

    story.append(p("7.4 Linux ABI 覆盖结构", styles["h2"]))
    story.append(
        p(
            "RISC-V syscall 入口按文件、内存、进程、资源、信号、同步、时间、网络和设备拆分。初赛适配不仅增加 syscall 编号，也处理结构体布局、flag、errno、短读写、阻塞与超时等语义。下表列出当前对官方测试最关键的能力类别。",
            styles["body"],
        )
    )
    abi_rows = [
        [table_cell("类别", styles, True), table_cell("代表能力", styles, True), table_cell("关联测试", styles, True), table_cell("注意事项", styles, True)],
        [table_cell("文件与路径", styles), table_cell("openat/read/write/stat/getdents/rename/truncate/link", styles), table_cell("basic、BusyBox、LTP", styles), table_cell("dirfd、flag、软链接与错误码", styles)],
        [table_cell("向量与复制", styles), table_cell("readv/writev/preadv/pwritev/sendfile", styles), table_cell("LTP、libcbench", styles), table_cell("IOV_MAX、短读写与 offset 更新", styles)],
        [table_cell("内存", styles), table_cell("brk/mmap/munmap/mprotect/shm", styles), table_cell("basic、libctest", styles), table_cell("页对齐、权限、COW 与用户指针", styles)],
        [table_cell("进程", styles), table_cell("clone/fork/execve/wait4/exit", styles), table_cell("basic、lmbench、LTP", styles), table_cell("父子资源、退出状态与动态加载", styles)],
        [table_cell("时间", styles), table_cell("clock_gettime/nanosleep/timerfd/gettimeofday", styles), table_cell("libcbench、LTP", styles), table_cell("tv_nsec 校验与超时中断", styles)],
        [table_cell("同步与复用", styles), table_cell("futex/poll/select/pselect/eventfd/flock", styles), table_cell("libcbench、LTP", styles), table_cell("等待队列、nonblock 与 wake 语义", styles)],
        [table_cell("信号", styles), table_cell("sigaction/sigprocmask/kill/sigreturn", styles), table_cell("libctest、LTP", styles), table_cell("mask、handler 与 trap 上下文", styles)],
        [table_cell("网络", styles), table_cell("socket/bind/connect/send/recv/poll", styles), table_cell("后续网络与应用", styles), table_cell("初赛 functional 优先级较低", styles)],
    ]
    story.append(make_table(abi_rows, [2.4 * cm, 6.0 * cm, 4.0 * cm, 4.4 * cm], compact=True))
    story.append(Spacer(1, 0.35 * cm))
    story.append(p("syscall 实现存在不代表所有边界 case 已兼容。LTP allowlist 只选择已经理解其前置条件、失败影响和验证方法的 case。", styles["quote"]))
    story.append(PageBreak())

    story.append(p("7.5 测试组推进策略", styles["h2"]))
    strategy_rows = [
        [table_cell("测试组", styles, True), table_cell("价值", styles, True), table_cell("当前策略", styles, True), table_cell("下一验收点", styles, True)],
        [table_cell("basic", styles), table_cell("启动、基础 syscall 与双 libc 基线", styles), table_cell("四列 102，作为每次回归门禁", styles), table_cell("保持满分且不增加耗时", styles)],
        [table_cell("BusyBox", styles), table_cell("文件系统与常用命令组合", styles), table_cell("私有工作目录，避免递归大树", styles), table_cell("四列稳定不回退", styles)],
        [table_cell("Lua", styles), table_cell("解释器、文件与内存综合场景", styles), table_cell("完整资源 staging 后运行脚本", styles), table_cell("四列 9 保持", styles)],
        [table_cell("libcbench", styles), table_cell("malloc、线程、futex 与性能路径", styles), table_cell("保留三列已计分，高风险项后置", styles), table_cell("补齐 glibc-la 且不截断", styles)],
        [table_cell("libctest", styles), table_cell("libc 语义密集、单项价值高", styles), table_cell("逐 case、静态/动态分开、独立 timeout", styles), table_cell("LoongArch 首批可识别 case", styles)],
        [table_cell("LTP", styles), table_cell("大规模 Linux ABI 验证", styles), table_cell("allowlist 分批扩展，失败继续", styles), table_cell("LoongArch 非零并提高 RV 稳定项", styles)],
        [table_cell("lmbench/iozone", styles), table_cell("性能与复杂文件路径", styles), table_cell("全部 functional 后置，严格限时", styles), table_cell("不影响前序分数后再扩大", styles)],
        [table_cell("网络/实时", styles), table_cell("应用与实时能力扩展", styles), table_cell("初赛末期不作为主攻方向", styles), table_cell("核心 ABI 稳定后再进入", styles)],
    ]
    story.append(make_table(strategy_rows, [2.3 * cm, 4.6 * cm, 5.7 * cm, 4.2 * cm], compact=True))
    story.append(Spacer(1, 0.45 * cm))
    story.append(p("7.5.1 提交批次原则", styles["h2"]))
    story.append(
        bullets(
            [
                "一次提交必须有清晰的目标测试组和可解释的预期增量。",
                "涉及构建、文件系统、执行顺序和 ABI 的改动分开提交，便于回退与归因。",
                "新增大批 case 前先确认 runner 格式与一个代表 case，避免整组无效。",
                "任何可能异常退出或长时间阻塞的测试必须有独立 timeout 和 END 收尾。",
            ],
            styles,
        )
    )
    story.append(PageBreak())

    story.append(p("7.6 后续里程碑与预期成果", styles["h2"]))
    milestone_rows = [
        [table_cell("阶段", styles, True), table_cell("重点工作", styles, True), table_cell("预期成果", styles, True)],
        [table_cell("M1 保分", styles), table_cell("冻结 1009.8903911255481 对应稳定执行顺序；保持双架构构建门禁", styles), table_cell("每次提交 basic/BusyBox/Lua 不回退", styles)],
        [table_cell("M2 LA functional", styles), table_cell("从首个 LoongArch libctest/LTP 日志补齐 syscall 与运行时", styles), table_cell("LA libctest/LTP 由 0 变为可持续增长", styles)],
        [table_cell("M3 资源语义", styles), table_cell("收敛 fd、fork/exec/wait、signal 与超时的跨架构差异", styles), table_cell("多进程和动态 libc case 更稳定", styles)],
        [table_cell("M4 性能路径", styles), table_cell("在 functional 完成后恢复 lmbench/iozone 小子集", styles), table_cell("性能组不再截断前序测试", styles)],
        [table_cell("M5 文档归档", styles), table_cell("整理来源、架构、验证矩阵和已知限制", styles), table_cell("提交材料与代码证据一致", styles)],
    ]
    story.append(make_table(milestone_rows, [3.0 * cm, 8.0 * cm, 5.8 * cm]))
    story.append(Spacer(1, 0.55 * cm))
    story.append(p("7.7 阶段性结论", styles["h2"]))
    story.append(
        p(
            "SWTC 已完成从“能够启动的两套内核”到“能够在同一官方提交中构建、发现测试资产、执行真实 Linux 用户程序并获得四列分项”的关键跨越。1009.8903911255481 分不是终点，而是一条具有工程证据的高分基线。后续工作的核心是提高 LoongArch functional 覆盖，并在不牺牲已有稳定性的前提下逐步扩大 Linux ABI 兼容范围。",
            styles["body"],
        )
    )
    story.append(p("项目现阶段最重要的资产，是已经建立的双架构交付链、可重复验证流程以及能够从真实日志解释得分变化的工程方法。", styles["quote"]))
    story.append(PageBreak())

    # 8. References
    story.append(p("8. 参考、开源基础与致谢", styles["h1"]))
    story.append(
        p(
            "SWTC 在遵守原项目许可证的前提下学习并复用开源操作系统的架构基础。RISC-V 主线以 Titanix 往届 GPLv3 开源项目为基础；LoongArch 主线参考 StarryX 与 ArceOS 组件化设计。本队增量主要集中在 2026 官方评测适配、双架构构建、Linux ABI 修复、测试调度和离线构建工程化。",
            styles["body"],
        )
    )
    refs = [
        ("Titanix", "https://github.com/greenhandzpx/Titanix"),
        ("StarryX", "https://github.com/Anekoique/StarryX"),
        ("NoAxiom-OS", "https://github.com/NoAxiom/NoAxiom-OS"),
        ("NPUcore-IMPACT", "https://github.com/Fediory/NPUcore-IMPACT"),
        ("UMI", "https://github.com/js2xxx/umi"),
        ("Alien", "https://github.com/Godones/Alien"),
        ("NPUcore", "https://github.com/echodjx/NPUcore"),
        ("UltraOS", "https://github.com/xiyurain/UltraOS"),
        ("Del0n1x / myOS", "https://github.com/Ya0rk/myOS"),
        ("Chronix", "https://github.com/PACTHEMAN123/Chronix"),
    ]
    for index, (name, url) in enumerate(refs, 1):
        story.append(p(f"[{index}] {name}：<link href=\"{url}\" color=\"#000000\">{url}</link>", styles["reference"]))
    story.append(Spacer(1, 0.45 * cm))
    story.append(p("开源与原创性说明", styles["h2"]))
    story.append(
        p(
            "项目保留所使用开源组件的许可证和来源说明，不直接复制未授权实现。文档所述比赛适配、测试组织、构建门禁和 ABI 修复以本仓库提交记录与验证材料为依据。",
            styles["body"],
        )
    )
    story.append(Spacer(1, 1.1 * cm))
    story.append(p("— 文档结束 —", styles["cover_subtitle"]))
    return story


def generate() -> Path:
    register_fonts()
    styles = build_styles()

    doc = SWTCDocTemplate(
        str(OUTPUT_PDF),
        pagesize=A4,
        leftMargin=MARGIN_X,
        rightMargin=MARGIN_X,
        topMargin=MARGIN_TOP,
        bottomMargin=MARGIN_BOTTOM,
        title="SWTC 初赛作品文档",
        author="上海电力大学 sudo_win_the_cscc",
        subject="2026 操作系统设计赛内核实现赛道初赛文档",
        creator="SWTC PDF generator",
    )
    frame = Frame(
        doc.leftMargin,
        doc.bottomMargin,
        doc.width,
        doc.height,
        id="body",
        leftPadding=0,
        rightPadding=0,
        topPadding=0,
        bottomPadding=0,
    )
    doc.addPageTemplates([PageTemplate(id="main", frames=[frame], onPage=add_page_number)])
    doc.multiBuild(build_story(styles))
    return OUTPUT_PDF


if __name__ == "__main__":
    result = generate()
    print(result)
