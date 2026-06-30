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
    Spacer,
    Table,
    TableStyle,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = REPO_ROOT / "docs" / "assets"
OUTPUT_DIR = REPO_ROOT / "output" / "pdf"
OUTPUT_PDF = OUTPUT_DIR / "SWTC-preliminary-report.pdf"

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
TEXT = colors.HexColor("#20242A")


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
            textColor=NAVY,
        ),
        "cover_title": ParagraphStyle(
            "CoverTitle",
            parent=base["Title"],
            fontName="SWTCCN-Bold",
            fontSize=29,
            leading=38,
            alignment=TA_CENTER,
            textColor=DEEP_RED,
        ),
        "cover_subtitle": ParagraphStyle(
            "CoverSubtitle",
            parent=base["Normal"],
            fontName="SWTCCN",
            fontSize=13,
            leading=20,
            alignment=TA_CENTER,
            textColor=MID_GRAY,
        ),
        "h1": ParagraphStyle(
            "H1",
            parent=base["Heading1"],
            fontName="SWTCCN-Bold",
            fontSize=19,
            leading=25,
            spaceBefore=0,
            spaceAfter=13,
            textColor=DEEP_RED,
        ),
        "h2": ParagraphStyle(
            "H2",
            parent=base["Heading2"],
            fontName="SWTCCN-Bold",
            fontSize=13.5,
            leading=19,
            spaceBefore=8,
            spaceAfter=7,
            textColor=NAVY,
        ),
        "body": ParagraphStyle(
            "Body",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=10.2,
            leading=17,
            alignment=TA_LEFT,
            firstLineIndent=2 * 10.2,
            textColor=TEXT,
            spaceAfter=6,
        ),
        "body_no_indent": ParagraphStyle(
            "BodyNoIndent",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=10.2,
            leading=16,
            alignment=TA_LEFT,
            textColor=TEXT,
            spaceAfter=5,
        ),
        "small": ParagraphStyle(
            "Small",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=13,
            textColor=MID_GRAY,
        ),
        "table": ParagraphStyle(
            "TableText",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=12.5,
            textColor=TEXT,
        ),
        "table_center": ParagraphStyle(
            "TableCenter",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.6,
            leading=12.5,
            alignment=TA_CENTER,
            textColor=TEXT,
        ),
        "caption": ParagraphStyle(
            "Caption",
            parent=base["BodyText"],
            fontName="SWTCCN",
            fontSize=8.8,
            leading=13,
            alignment=TA_CENTER,
            textColor=MID_GRAY,
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
            textColor=TEXT,
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
            textColor=TEXT,
            spaceAfter=4,
        ),
    }


def p(text: str, style: ParagraphStyle) -> Paragraph:
    return Paragraph(text, style)


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
                ("BACKGROUND", (0, 0), (-1, 0), NAVY),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.white),
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
                fillColor=TEXT,
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
    d.add(String(8, 237, "统一构建层", fontName="SWTCCN-Bold", fontSize=9, fillColor=DEEP_RED))
    add_box(d, 20, 190, 105, 38, ["GitLab main", "统一源码入口"], PALE_BLUE)
    add_box(d, 176, 190, 105, 38, ["根目录 make all", "离线构建与门禁"], PALE_RED)
    add_box(d, 330, 211, 108, 34, ["kernel-rv", "RISC-V64 ELF"], colors.HexColor("#E8F3EE"))
    add_box(d, 330, 166, 108, 34, ["kernel-la", "LoongArch64 ELF"], colors.HexColor("#FFF3DF"))
    add_arrow(d, 125, 209, 176, 209)
    d.add(Line(281, 209, 305, 209, strokeColor=MID_GRAY, strokeWidth=1.2))
    d.add(Line(305, 183, 305, 228, strokeColor=MID_GRAY, strokeWidth=1.2))
    add_arrow(d, 305, 228, 330, 228)
    add_arrow(d, 305, 183, 330, 183)

    d.add(String(8, 145, "官方运行层", fontName="SWTCCN-Bold", fontSize=9, fillColor=DEEP_RED))
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

    d.add(String(8, 75, "内核运行路径", fontName="SWTCCN-Bold", fontSize=9, fillColor=DEEP_RED))
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
    canvas.setFillColor(MID_GRAY)
    canvas.drawString(MARGIN_X, PAGE_HEIGHT - 0.95 * cm, "SWTC 初赛作品文档")
    canvas.drawRightString(PAGE_WIDTH - MARGIN_X, PAGE_HEIGHT - 0.95 * cm, "上海电力大学 · sudo_win_the_cscc")
    canvas.setStrokeColor(LIGHT_LINE)
    canvas.line(MARGIN_X, 1.05 * cm, PAGE_WIDTH - MARGIN_X, 1.05 * cm)
    canvas.drawCentredString(PAGE_WIDTH / 2, 0.65 * cm, f"第 {doc.page} 页")
    canvas.restoreState()


def add_page_number(canvas, doc) -> None:
    page_header_footer(canvas, doc)


def build_story(styles: dict[str, ParagraphStyle]) -> list[object]:
    logo_path = ASSET_DIR / "shanghai-university-of-electric-power-logo.png"
    evaluation_path = ASSET_DIR / "official-evaluation-2026-06-30.png"

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
                ("TEXTCOLOR", (0, 0), (0, -1), DEEP_RED),
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
    story.append(architecture_diagram())
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
            "初赛阶段优先保证官方入口、Linux ABI 和 functional 测试链路，不将网络、图形界面等展示性功能置于计分闭环之前。对于尚未稳定的 iozone、lmbench 和大规模 LTP，采用后置、分批和可回滚策略，避免一次改动破坏已验证基线。",
            styles["body"],
        )
    )
    story.append(p("创新重点不在堆叠功能数量，而在双架构、真实测试资产和受限评测环境下建立稳定、可解释、可复现的内核运行链路。", styles["quote"]))
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
            "评测成绩是阶段性工程证据，不等同于完整 Linux 兼容度。项目仍通过本地官方镜像回归和线上分项日志交叉验证每项能力。",
            styles["quote"],
        )
    )
    story.append(PageBreak())

    # 7. Build and roadmap
    story.append(p("7. 构建、验证与后续计划", styles["h1"]))
    story.append(p("7.1 构建方式", styles["h2"]))
    build_box = Table(
        [[p("<font color=\"#FFFFFF\">cd /mnt/e/WorkSpace/MinOS/os-kernel-contest<br/>make all</font>", styles["body_no_indent"])]],
        colWidths=[16.8 * cm],
    )
    build_box.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, -1), colors.HexColor("#202A35")),
                ("TEXTCOLOR", (0, 0), (-1, -1), colors.white),
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
        [table_cell("P1", styles, True), table_cell("恢复并巩固已验证高分路径", styles), table_cell("basic/BusyBox/Lua/libcbench 分项不回退", styles)],
        [table_cell("P2", styles, True), table_cell("扩展 LA libctest 与 LTP", styles), table_cell("逐 case 可解释，新增 syscall 有本地和线上证据", styles)],
        [table_cell("P3", styles, True), table_cell("完善进程与文件描述符模型", styles), table_cell("fork/exec/wait 与多程序资源隔离回归通过", styles)],
    ]
    story.append(make_table(roadmap_rows, [2.0 * cm, 6.2 * cm, 8.6 * cm]))
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
        story.append(p(f"[{index}] {name}：<link href=\"{url}\" color=\"#2F75B5\">{url}</link>", styles["reference"]))
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
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    doc = BaseDocTemplate(
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
    doc.build(build_story(styles))
    return OUTPUT_PDF


if __name__ == "__main__":
    result = generate()
    print(result)
