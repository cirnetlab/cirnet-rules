#pragma once

#include "governance_editor.h"
#include "theme.h"

#include <QApplication>
#include <QComboBox>
#include <QDialog>
#include <QEnterEvent>
#include <QFontDatabase>
#include <QLabel>
#include <QLineEdit>
#include <QMenu>
#include <QPlainTextEdit>
#include <QScrollArea>
#include <QScrollBar>
#include <QScopeGuard>
#include <QSlider>
#include <QSpinBox>
#include <QPushButton>
#include <QTest>
#include <QTextDocument>
#include <QTextLayout>
#include <QVBoxLayout>
#include <QWheelEvent>

namespace shared_polish {
inline void scrollbarsRevealOnThumbHoverAndWheel()
{
    ThemeManager theme;
    theme.apply("dark", "comfortable", 100);
    QScrollArea scroll;
    auto *canvas = new QWidget;
    canvas->resize(550, 550);
    scroll.setWidget(canvas);
    scroll.resize(260, 210);
    scroll.show();
    QTRY_VERIFY(scroll.horizontalScrollBar()->isVisible() && scroll.verticalScrollBar()->isVisible());
    const auto area = scroll.viewport()->geometry();
    QEvent leave(QEvent::Leave);
    QApplication::sendEvent(scroll.verticalScrollBar(), &leave);
    QApplication::sendEvent(scroll.horizontalScrollBar(), &leave);
    const auto hiddenVertical = scroll.verticalScrollBar()->grab().toImage();
    const auto hiddenHorizontal = scroll.horizontalScrollBar()->grab().toImage();
    QEnterEvent enter(QPointF(8, 8), QPointF(8, 8), scroll.mapToGlobal(QPoint(8, 8)));
    QApplication::sendEvent(&scroll, &enter);
    QCOMPARE(scroll.verticalScrollBar()->grab().toImage(), hiddenVertical);
    QCOMPARE(scroll.horizontalScrollBar()->grab().toImage(), hiddenHorizontal);
    QApplication::sendEvent(scroll.horizontalScrollBar(), &enter);
    QVERIFY(scroll.horizontalScrollBar()->grab().toImage() != hiddenHorizontal);
    QCOMPARE(scroll.viewport()->geometry(), area);
    QApplication::sendEvent(scroll.horizontalScrollBar(), &leave);
    QCOMPARE(scroll.horizontalScrollBar()->grab().toImage(), hiddenHorizontal);

    auto *viewport = scroll.viewport();
    const auto origin = viewport->rect().center();
    QWheelEvent wheel(origin, viewport->mapToGlobal(origin), {}, QPoint(0, -120), Qt::NoButton,
                      Qt::NoModifier, Qt::NoScrollPhase, false);
    QApplication::sendEvent(viewport, &wheel);
    QVERIFY(scroll.verticalScrollBar()->value() > 0);
    QVERIFY(scroll.verticalScrollBar()->grab().toImage() != hiddenVertical);
    QCOMPARE(scroll.horizontalScrollBar()->grab().toImage(), hiddenHorizontal);
    QTRY_VERIFY_WITH_TIMEOUT(scroll.verticalScrollBar()->grab().toImage() == hiddenVertical, 1800);
}

inline void appearancePixelsAndPaletteApplyToOpenWidgets()
{
    ThemeManager theme;
    theme.apply("dark", "comfortable", 100);
    QWidget frame;
    auto *layout = new QVBoxLayout(&frame);
    auto *workspace = new QLabel("Workspace");
    auto *settings = new QLabel("Settings");
    auto *conversation = new QPlainTextEdit;
    conversation->setObjectName("conversationComposer");
    auto *code = new QPlainTextEdit;
    code->setProperty("codeText", true);
    for (auto *widget : {static_cast<QWidget *>(workspace), static_cast<QWidget *>(settings),
                         static_cast<QWidget *>(conversation), static_cast<QWidget *>(code)})
        layout->addWidget(widget);
    QMenu menu(&frame);
    auto *action = menu.addAction("Menu action");
    frame.show();
    menu.ensurePolished();
    const auto menuHeight = menu.actionGeometry(action).height();
    theme.apply("dark", "comfortable", 100, {
        {"interface_px", 21}, {"conversation_px", 17}, {"code_px", 15}, {"density_px", 10},
        {"background", "#203040"}, {"foreground", "#E0E6F2"},
        {"accent", "custom"}, {"custom_accent", "#A367DD"}
    });
    QCOMPARE(QApplication::font().pixelSize(), 21);
    QCOMPARE(workspace->font().pixelSize(), 21);
    QCOMPARE(settings->font().pixelSize(), 21);
    QCOMPARE(menu.font().pixelSize(), 21);
    QVERIFY(menu.actionGeometry(action).height() > menuHeight);
    QCOMPARE(conversation->font().pixelSize(), 17);
    QCOMPARE(code->font().pixelSize(), 15);
    QCOMPARE(QApplication::palette().color(QPalette::Window), QColor("#203040"));
    QCOMPARE(QApplication::palette().color(QPalette::WindowText), QColor("#E0E6F2"));
    QCOMPARE(QApplication::palette().color(QPalette::Highlight), QColor("#A367DD"));
    for (const auto &family : QFontDatabase::families()) {
        if (family.startsWith(u'@') || family == code->font().family()
            || !QFontDatabase::isSmoothlyScalable(family))
            continue;
        theme.apply("dark", "comfortable", 100, {{"font_family", family}});
        QCOMPARE(workspace->font().family(), family);
        QCOMPARE(menu.font().family(), family);
        QCOMPARE(conversation->document()->defaultFont().family(), family);
        QCOMPARE(code->document()->defaultFont().family(), family);
        break;
    }
    theme.apply("dark", "comfortable", 100);
}

inline void customColorsKeepOpenSurfacesReadable()
{
    ThemeManager theme;
    const auto restore = qScopeGuard([&] { theme.apply("dark", "comfortable", 100); });
    QDialog panel;
    auto *layout = new QVBoxLayout(&panel);
    auto *label = new QLabel("Settings");
    auto *field = new QLineEdit;
    field->setPlaceholderText("Search");
    auto *editor = new QPlainTextEdit;
    editor->setObjectName("conversationComposer");
    auto *primary = new QPushButton("Apply");
    primary->setProperty("primary", true);
    layout->addWidget(label);
    layout->addWidget(field);
    layout->addWidget(editor);
    layout->addWidget(primary);
    QMenu menu(&panel);
    menu.addAction("Menu action");
    menu.ensurePolished();
    panel.resize(350, 235);
    panel.show();

    const auto colorAt = [](QWidget &widget, const QPoint &point) {
        const auto frame = widget.grab();
        const auto dpr = frame.devicePixelRatio();
        return frame.toImage().pixelColor(qRound(point.x() * dpr), qRound(point.y() * dpr));
    };
    QColor previousEditor;
    QColor previousPanel;
    QColor previousMenu;
    for (const auto *background : {"#10172C", "#757575", "#777777", "#A9B4EF", "#FFFDE0"}) {
        theme.apply("dark", "comfortable", 100, {
            {"background", background}, {"foreground", background},
            {"accent", "custom"}, {"custom_accent", background}
        });
        const auto colors = QApplication::palette();
        QCOMPARE(colors.color(QPalette::Window), QColor(background));
        for (const auto group : {QPalette::Active, QPalette::Inactive}) {
            for (const auto role : {QPalette::Window, QPalette::Base, QPalette::Button,
                                    QPalette::AlternateBase, QPalette::Midlight}) {
                QVERIFY2(appearanceColorContrast(colors.color(group, QPalette::Text),
                                                colors.color(group, role)) >= 4.5, background);
            }
            QVERIFY2(appearanceColorContrast(colors.color(group, QPalette::HighlightedText),
                                            colors.color(group, QPalette::Highlight)) >= 4.5, background);
        }
        const auto dialog = colorAt(panel, QPoint(4, panel.height() / 2));
        const auto composition = colorAt(*editor->viewport(), editor->viewport()->rect().center());
        const auto input = colorAt(*field, QPoint(field->width() - 14, field->height() / 2));
        const auto action = colorAt(*primary, QPoint(12, primary->height() / 2));
        menu.resize(menu.sizeHint());
        const auto popup = colorAt(menu, QPoint(menu.width() - 6, menu.height() / 2));
        QVERIFY2(dialog != previousPanel && composition != previousEditor && popup != previousMenu, background);
        QVERIFY2(appearanceColorContrast(label->palette().color(QPalette::WindowText), dialog) >= 4.5, background);
        QVERIFY2(appearanceColorContrast(colors.color(QPalette::Text), composition) >= 4.5, background);
        QVERIFY2(appearanceColorContrast(colors.color(QPalette::PlaceholderText), input) >= 4.5, background);
        QVERIFY2(appearanceColorContrast(colors.color(QPalette::HighlightedText), action) >= 4.5, background);
        QVERIFY2(appearanceColorContrast(colors.color(QPalette::Text), popup) >= 4.5, background);
        previousEditor = composition;
        previousPanel = dialog;
        previousMenu = popup;
    }
}

inline void governanceTextTracksLiveTheme()
{
    ThemeManager theme;
    theme.apply("dark", "comfortable", 100);
    const auto restore = qScopeGuard([&] { theme.apply("dark", "comfortable", 100); });
    GovernanceEditor editor;
    GovernancePreview preview;
    editor.setPlainText("# Header\nA draft");
    editor.selectAll();
    preview.showSource("[Policy](policy.md)", true);
    const auto ink = [](const QTextDocument &document) {
        const auto first = document.firstBlock();
        const auto formats = first.layout()->formats();
        return formats.isEmpty() ? first.begin().fragment().charFormat().foreground().color()
                                 : formats.first().format.foreground().color();
    };
    QTRY_COMPARE(ink(*editor.document()), editor.palette().color(QPalette::Link));
    QCOMPARE(ink(*preview.document()), preview.palette().color(QPalette::Link));

    theme.apply("light", "comfortable", 100, {{"accent", "purple"}});
    const auto verify = [&](const auto &widget) {
        QCOMPARE(ink(*widget.document()), widget.palette().color(QPalette::Link));
    };
    verify(editor);
    verify(preview);
    QCOMPARE(editor.toPlainText(), QStringLiteral("# Header\nA draft"));
    QVERIFY(editor.textCursor().hasSelection());
    QCOMPARE(preview.document()->firstBlock().begin().fragment().charFormat().anchorHref(), QStringLiteral("policy.md"));
}

inline void wheelScrollsWithoutChangingEditors()
{
    QScrollArea scroll;
    auto *canvas = new QWidget;
    auto *layout = new QVBoxLayout(canvas);
    auto *spin = new QSpinBox;
    auto *combo = new QComboBox;
    auto *slider = new QSlider(Qt::Horizontal);
    spin->setValue(40);
    combo->addItems({"First", "Second", "Third"});
    slider->setValue(40);
    for (auto *editor : {static_cast<QWidget *>(spin), static_cast<QWidget *>(combo),
                         static_cast<QWidget *>(slider)})
        layout->addWidget(editor);
    auto *inner = new QScrollArea;
    auto *nestedSpin = new QSpinBox;
    nestedSpin->setValue(25);
    inner->setWidget(nestedSpin);
    inner->setWidgetResizable(true);
    inner->setFixedHeight(42);
    layout->addWidget(inner);
    layout->addStretch(1);
    canvas->setMinimumHeight(700);
    scroll.setWidgetResizable(true);
    scroll.setWidget(canvas);
    scroll.resize(310, 230);
    scroll.show();
    scroll.activateWindow();
    QTRY_VERIFY(scroll.verticalScrollBar()->maximum() > 0);
    const auto wheel = [&](QWidget *editor) {
        scroll.verticalScrollBar()->setValue(0);
        editor->setFocus();
        const auto local = editor->rect().center();
        QWheelEvent event(local, editor->mapToGlobal(local), {}, QPoint(0, -120), Qt::NoButton,
                          Qt::NoModifier, Qt::NoScrollPhase, false);
        QApplication::sendEvent(editor, &event);
    };
    wheel(spin);
    QCOMPARE(spin->value(), 40);
    QVERIFY(scroll.verticalScrollBar()->value() > 0);
    wheel(combo);
    QCOMPARE(combo->currentIndex(), 0);
    QVERIFY(scroll.verticalScrollBar()->value() > 0);
    wheel(slider);
    QCOMPARE(slider->value(), 40);
    QVERIFY(scroll.verticalScrollBar()->value() > 0);
    QCOMPARE(inner->verticalScrollBar()->maximum(), 0);
    wheel(nestedSpin);
    QCOMPARE(nestedSpin->value(), 25);
    QVERIFY(scroll.verticalScrollBar()->value() > 0);
    QTest::keyClick(spin, Qt::Key_Up);
    QCOMPARE(spin->value(), 41);
}
}
