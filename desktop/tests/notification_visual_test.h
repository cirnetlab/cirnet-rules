#pragma once

#include "interaction_cases.h"
#include "main_window.h"
#include "notification_center.h"

#include <QLabel>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>

namespace notification_visual {

inline void verifyAlignmentAndSeverities()
{
    ThemeManager theme;
    const struct {
        QString severity;
        QString message;
        int hue;
    } cases[]{
        {"success", "Repository binding saved.", 145},
        {"info", "Run 3 · QUEUED", 210},
        {"warning", "Approval is required.", 33},
        {"error", "Runtime connection interrupted. Reconnect to restore saved state and review the recorded errors.", 4},
    };
    for (const auto &mode : {QStringLiteral("dark"), QStringLiteral("light")}) {
        theme.apply(mode, "comfortable", mode == "light" ? 125 : 100);
        QWidget host;
        host.resize(560, 320);
        NotificationCenter notices(host);
        notices.setReducedMotion(true);
        interaction_cases::pinToPrimary(host);
        host.show();
        for (const auto &[severity, message, hue] : cases) {
            notices.post(message, severity);
            QPointer<QWidget> card = host.findChild<QWidget *>("toast");
            QTRY_VERIFY(card && card->isVisible());
            auto *icon = card->findChild<QLabel *>("toastIcon");
            auto *body = card->findChild<QLabel *>("toastBody");
            auto *close = card->findChild<QToolButton *>("dismissToast");
            QVERIFY(icon && body && close);
            const auto bodyY = body->geometry().center().y();
            QVERIFY2(qAbs(icon->geometry().center().y() - bodyY) <= 1,
                     "The severity glyph must align with the message vertically");
            QVERIFY2(qAbs(close->geometry().center().y() - bodyY) <= 1,
                     "The dismiss control must align with the message vertically");
            QVERIFY(icon->geometry().right() < body->geometry().left());
            QVERIFY(body->geometry().right() < close->geometry().left());
            QVERIFY(close->height() >= 28 && close->width() >= 28);
            if (severity == "error")
                QVERIFY(body->text().contains(u'\n'));

            const auto pixels = icon->grab().toImage();
            int severityInk = 0;
            for (int y = 0; y < pixels.height(); ++y) {
                for (int x = 0; x < pixels.width(); ++x) {
                    const auto color = pixels.pixelColor(x, y);
                    const auto distance = qAbs(color.hsvHue() - hue);
                    if (color.alpha() >= 128 && color.hsvSaturation() >= 80 &&
                        qMin(distance, 360 - distance) <= 22)
                        ++severityInk;
                }
            }
            QVERIFY2(severityInk >= 8, "A legible severity-specific color must be visible in the glyph");
            interaction_cases::capture(*card, "toast-" + severity + "-" + mode);
            QTest::keyClick(close, Qt::Key_Space);
            QTRY_VERIFY(!card || !card->isVisible());
            QTRY_VERIFY(!card);
        }
    }
    theme.apply("dark", "comfortable", 100);
}

inline void verifyConversationPlacement(MainWindow &window)
{
    auto *center = window.findChild<NotificationCenter *>();
    auto *history = window.findChild<QListWidget *>("conversationList");
    auto *composer = window.findChild<QPlainTextEdit *>("conversationComposer");
    auto *actions = window.findChild<QPushButton *>("conversationActions");
    auto *automations = window.findChild<QPushButton *>("navAutomations");
    auto *settings = window.findChild<QToolButton *>("navSettings");
    auto *back = window.findChild<QPushButton *>("settingsBack");
    QVERIFY(center && history && composer && actions && automations && settings && back);
    QTRY_VERIFY(history->count() > 0);
    QVERIFY(automations->isChecked());

    center->post(QStringLiteral("Run 5 · QUEUED"), QStringLiteral("info"));
    QPointer<QWidget> carried = window.findChild<QWidget *>("toast");
    QTRY_VERIFY(carried && carried->isVisible());
    auto *host = carried->parentWidget();
    const auto isLowerRight = [host](const QWidget *card) {
        return host->rect().bottom() - card->geometry().bottom() <= 20
            && host->rect().right() - card->geometry().right() <= 20;
    };
    const auto isUpperRight = [host](const QWidget *card) {
        return card->geometry().top() >= 0 && card->geometry().top() <= 100
            && host->rect().right() - card->geometry().right() <= 20;
    };
    const auto bounds = [host](const QWidget *widget) {
        return QRect(widget->mapTo(host, QPoint{}), widget->size());
    };
    QVERIFY(isLowerRight(carried));
    interaction_cases::capture(window, "toast-automations-lower-right");

    QTest::mouseClick(history->viewport(), Qt::LeftButton, Qt::NoModifier,
                      history->visualItemRect(history->item(0)).center());
    QTRY_VERIFY(composer->isVisible() && !composer->isReadOnly() && actions->isVisible());
    QVERIFY(carried && carried->isVisible());
    QVERIFY2(isUpperRight(carried), "Entering Conversation must move an existing toast to the upper right");
    QTRY_VERIFY(carried && !carried->geometry().intersects(bounds(actions)));
    QVERIFY(!carried->geometry().intersects(bounds(composer)));
    QTest::mouseClick(composer->viewport(), Qt::LeftButton, Qt::NoModifier, QPoint(8, 8));
    QVERIFY(carried && carried->isVisible() && isUpperRight(carried));
    if (window.isActiveWindow())
        QVERIFY(composer->hasFocus());
    interaction_cases::capture(window, "toast-conversation-upper-right");

    center->post(QStringLiteral("Approval is required."), QStringLiteral("warning"));
    QWidget *newest = nullptr;
    for (auto *card : window.findChildren<QWidget *>("toast")) {
        if (card != carried)
            newest = card;
    }
    QVERIFY(newest && newest->isVisible());
    QVERIFY(isUpperRight(newest));
    QVERIFY(newest->geometry().bottom() < carried->geometry().top());
    QTest::mouseClick(settings, Qt::LeftButton);
    QTRY_VERIFY(isLowerRight(newest));
    QTest::mouseClick(back, Qt::LeftButton);
    QTRY_VERIFY(isUpperRight(newest));
    QTest::mouseClick(automations, Qt::LeftButton);
    QTRY_VERIFY(isLowerRight(newest));
    QVERIFY(newest->geometry().top() > carried->geometry().bottom());
}

} // namespace notification_visual
