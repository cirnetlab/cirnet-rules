#pragma once

#include "theme.h"

#include <QApplication>
#include <QComboBox>
#include <QEnterEvent>
#include <QLabel>
#include <QMenu>
#include <QPlainTextEdit>
#include <QScrollArea>
#include <QScrollBar>
#include <QSlider>
#include <QSpinBox>
#include <QTest>
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
    theme.apply("dark", "comfortable", 100);
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
