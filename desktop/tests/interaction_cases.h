#pragma once

#include "theme.h"

#include <QAbstractItemView>
#include <QApplication>
#include <QComboBox>
#include <QImage>
#include <QMenu>
#include <QPushButton>
#include <QScreen>
#include <QSignalSpy>
#include <QSpinBox>
#include <QStyleOption>
#include <QTest>
#include <QTimeEdit>
#include <QToolButton>
#include <QVBoxLayout>
#include <QWheelEvent>

namespace interaction_cases {

inline void pinToPrimary(QWidget &widget)
{
    if (qEnvironmentVariableIsSet("OPENRULES_TEST_PRIMARY_SCREEN")) {
        auto *screen = QGuiApplication::primaryScreen();
        widget.setScreen(screen);
        widget.move(screen->availableGeometry().topLeft() + QPoint(8, 8));
    }
}

inline void capture(QWidget &widget, const QString &name)
{
    const auto directory = qEnvironmentVariable("OPENRULES_UI_CAPTURE");
    if (!directory.isEmpty())
        QVERIFY(widget.grab().save(directory + "/" + name + ".png"));
}

inline int inkPixels(const QWidget &widget, const QImage &image, const QRect &area, const QColor &ink)
{
    const auto dpr = widget.devicePixelRatioF();
    const auto pixels = QRect(qRound(area.x() * dpr), qRound(area.y() * dpr),
                              qRound(area.width() * dpr), qRound(area.height() * dpr)).intersected(image.rect());
    int count = 0;
    for (int y = pixels.top(); y <= pixels.bottom(); ++y) {
        for (int x = pixels.left(); x <= pixels.right(); ++x) {
            const auto c = image.pixelColor(x, y);
            if (qAbs(c.red() - ink.red()) + qAbs(c.green() - ink.green()) + qAbs(c.blue() - ink.blue()) < 65)
                ++count;
        }
    }
    return count;
}

inline void mouseAndKeyboardFocus()
{
    ThemeManager theme;
    theme.apply("dark", "comfortable", 100);
    QWidget panel;
    auto *layout = new QVBoxLayout(&panel);
    auto *action = new QPushButton("New Chat", &panel);
    action->setProperty("navigation", true);
    auto *primary = new QPushButton("Add Automation", &panel);
    primary->setProperty("primary", true);
    layout->addWidget(action);
    layout->addWidget(primary);
    panel.resize(340, 150);
    pinToPrimary(panel);
    panel.show();
    panel.activateWindow();
    QSignalSpy clicked(action, &QPushButton::clicked);
    QTest::mousePress(action, Qt::LeftButton);
    capture(panel, "controls-action-pressed");
    QTest::mouseRelease(action, Qt::LeftButton);
    QTest::mouseMove(&panel, QPoint(2, 2));
    QCOMPARE(clicked.count(), 1);
    QTRY_VERIFY(action->hasFocus());
    capture(panel, "controls-action-released");
    const QColor accent = action->palette().color(QPalette::Highlight);
    const auto borderBand = QRect(10, 0, action->width() - 20, 3);
    const int mouseOutline = inkPixels(*action, action->grab().toImage(), borderBand, accent);
    QTest::keyClick(action, Qt::Key_Tab);
    QTRY_VERIFY(primary->hasFocus());
    QTest::keyClick(primary, Qt::Key_Tab, Qt::ShiftModifier);
    QTRY_VERIFY(action->hasFocus());
    capture(panel, "controls-action-keyboard-focus");
    const int keyboardOutline = inkPixels(*action, action->grab().toImage(), borderBand, accent);
    const auto primaryImage = primary->grab().toImage();
    const QRect primarySurface(8, primary->height() / 2 - 2, 6, 4);
    QVERIFY2(mouseOutline < 5, "Clicking and moving away must not leave an accent outline");
    QVERIFY2(keyboardOutline >= 8, "Tab focus must have a visible outline");
    QVERIFY2(inkPixels(*primary, primaryImage, primarySurface, accent) >= 5, "Primary action must retain its filled surface");
}

inline void iconStyles()
{
    ThemeManager theme;
    theme.apply("dark", "comfortable", 100);
    const auto reload = QApplication::style()->standardIcon(QStyle::SP_BrowserReload);
    const auto clear = QApplication::style()->standardIcon(QStyle::SP_LineEditClearButton);
    QVERIFY(!reload.isNull());
    QVERIFY(!clear.isNull());

    for (const auto &mode : {QStringLiteral("dark"), QStringLiteral("light")}) {
        theme.apply(mode, "comfortable", 100);
        const auto clearImage = clear.pixmap(QSize(20, 20)).toImage();
        const auto clearInk = clearImage.pixelColor(clearImage.rect().center());
        const auto muted = QApplication::palette().color(QPalette::PlaceholderText);
        QVERIFY(clearInk.alpha() >= 150);
        QVERIFY(qAbs(clearInk.red() - muted.red()) + qAbs(clearInk.green() - muted.green()) +
                qAbs(clearInk.blue() - muted.blue()) < 18);
        for (const auto dpr : {1.0, 1.25, 1.5, 2.0}) {
            const QSize logical(20, 20);
            const auto pixmap = reload.pixmap(logical, dpr);
            QCOMPARE(pixmap.size(), logical * dpr);
            QCOMPARE(pixmap.devicePixelRatio(), dpr);
            const auto image = pixmap.toImage();
            const auto color = QApplication::palette().color(QPalette::WindowText);
            QRect bounds;
            int foregroundPixels = 0;
            for (int y = 0; y < image.height(); ++y) {
                for (int x = 0; x < image.width(); ++x) {
                    const auto pixel = image.pixelColor(x, y);
                    if (pixel.alpha() < 150)
                        continue;
                    bounds |= QRect(x, y, 1, 1);
                    if (qAbs(pixel.red() - color.red()) + qAbs(pixel.green() - color.green()) +
                        qAbs(pixel.blue() - color.blue()) < 18)
                        ++foregroundPixels;
                }
            }
            QVERIFY2(foregroundPixels >= 18 * dpr * dpr, "Refresh ink must follow the current foreground palette");
            QVERIFY(bounds.width() >= 12 * dpr && bounds.height() >= 12 * dpr);
            QVERIFY(image.rect().adjusted(1, 1, -1, -1).contains(bounds));
            QVERIFY((bounds.center() - image.rect().center()).manhattanLength() <= 2 * dpr);
        }
    }
    theme.apply("dark", "comfortable", 100);
}

inline void compoundArrows()
{
    ThemeManager theme;
    for (const auto &mode : {QStringLiteral("dark"), QStringLiteral("light")}) {
        theme.apply(mode, "comfortable", 100);
        QWidget panel;
        auto *layout = new QVBoxLayout(&panel);
        auto *combo = new QComboBox(&panel);
        combo->addItems({"Daily", "Manual"});
        auto *spin = new QSpinBox(&panel);
        spin->setRange(85, 150);
        spin->setSingleStep(5);
        spin->setSuffix("%");
        spin->setValue(100);
        auto *time = new QTimeEdit(QTime(9, 15), &panel);
        time->setDisplayFormat("HH:mm");
        auto *menuButton = new QPushButton("Actions", &panel);
        auto *menu = new QMenu(menuButton);
        menu->addAction("Inspect");
        menuButton->setMenu(menu);
        for (auto *control : {static_cast<QWidget *>(combo), static_cast<QWidget *>(spin),
                              static_cast<QWidget *>(time), static_cast<QWidget *>(menuButton)})
            layout->addWidget(control);
        panel.resize(340, 240);
        pinToPrimary(panel);
        panel.show();
        panel.activateWindow();
        QCoreApplication::processEvents();
        capture(panel, "controls-arrows-" + mode);

        QStyleOptionComboBox comboStyle;
        comboStyle.initFrom(combo);
        const auto down = combo->style()->subControlRect(QStyle::CC_ComboBox, &comboStyle,
                                                          QStyle::SC_ComboBoxArrow, combo);
        auto spinRect = [](const QAbstractSpinBox &control, QStyle::SubControl part) {
            QStyleOptionSpinBox style;
            style.initFrom(&control);
            style.frame = control.hasFrame();
            style.buttonSymbols = control.buttonSymbols();
            return control.style()->subControlRect(QStyle::CC_SpinBox, &style, part, &control);
        };
        const auto up = spinRect(*spin, QStyle::SC_SpinBoxUp);
        const auto lower = spinRect(*spin, QStyle::SC_SpinBoxDown);
        const auto timeUp = spinRect(*time, QStyle::SC_SpinBoxUp);
        const auto timeDown = spinRect(*time, QStyle::SC_SpinBoxDown);
        QTest::mouseClick(combo, Qt::LeftButton, Qt::NoModifier, down.center());
        QTRY_VERIFY(combo->view()->isVisible());
        QTest::keyClick(combo->view(), Qt::Key_Escape);
        QTRY_VERIFY(!combo->view()->isVisible());
        QTest::mouseClick(spin, Qt::LeftButton, Qt::NoModifier, up.center());
        QCOMPARE(spin->value(), 105);
        QTest::keyClick(spin, Qt::Key_Down);
        QCOMPARE(spin->value(), 100);
        time->setCurrentSection(QDateTimeEdit::HourSection);
        QTest::mouseClick(time, Qt::LeftButton, Qt::NoModifier, timeUp.center());
        QCOMPARE(time->time(), QTime(10, 15));
        QTest::mouseClick(time, Qt::LeftButton, Qt::NoModifier, timeDown.center());
        QCOMPARE(time->time(), QTime(9, 15));
        menuButton->setFocus();
        QWheelEvent wheel(spin->rect().center(), spin->mapToGlobal(spin->rect().center()), {}, QPoint(0, 120),
                          Qt::NoButton, Qt::NoModifier, Qt::NoScrollPhase, false);
        QApplication::sendEvent(spin, &wheel);
        const bool wheelChangedUnfocusedValue = spin->value() != 100;
        spin->setValue(100);
        auto glyph = [](QWidget &control, const QRect &rect) {
            const QRect centered(rect.center() - QPoint(6, 4), QSize(13, 9));
            return inkPixels(control, control.grab().toImage(), centered,
                             control.palette().color(QPalette::ButtonText)) >= 3;
        };
        QVERIFY2(down.width() >= 24 && up.width() >= 24 && timeUp.width() >= 24,
                 "Compound controls need a distinct arrow hit area of at least 24 logical pixels");
        QVERIFY(glyph(*combo, down));
        QVERIFY(glyph(*spin, up));
        QVERIFY(glyph(*spin, lower));
        QVERIFY(glyph(*time, timeUp));
        QVERIFY(glyph(*time, timeDown));
        QVERIFY2(!wheelChangedUnfocusedValue, "Scrolling over an unfocused stepper must not change its value");
        spin->setValue(spin->maximum());
        QTest::mouseClick(spin, Qt::LeftButton, Qt::NoModifier, up.center());
        QCOMPARE(spin->value(), spin->maximum());
        QTest::mouseClick(spin, Qt::LeftButton, Qt::NoModifier, lower.center());
        QCOMPARE(spin->value(), spin->maximum() - spin->singleStep());
        time->setEnabled(false);
        QTest::mouseClick(time, Qt::LeftButton, Qt::NoModifier, timeUp.center());
        QCOMPARE(time->time(), QTime(9, 15));
        capture(panel, "controls-arrows-disabled-" + mode);
    }
}

} // namespace interaction_cases
