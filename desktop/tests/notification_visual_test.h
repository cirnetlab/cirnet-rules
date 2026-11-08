#pragma once

#include "interaction_cases.h"
#include "notification_center.h"

#include <QLabel>

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

} // namespace notification_visual
