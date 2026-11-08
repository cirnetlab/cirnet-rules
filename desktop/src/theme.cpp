#include "theme.h"

#include <QApplication>
#include <QFile>
#include <QPainter>
#include <QProxyStyle>
#include <QStyleFactory>
#include <QStyleHints>
#include <QStyleOption>
#include <QtMath>

namespace {
QFont systemFont;
QPalette systemPalette;

class CheckStyle final : public QProxyStyle {
public:
    CheckStyle() : QProxyStyle(QStyleFactory::create("Fusion")) {}

    int pixelMetric(PixelMetric metric, const QStyleOption *option, const QWidget *widget) const override
    {
        if (metric == PM_IndicatorWidth || metric == PM_IndicatorHeight)
            return 17;
        return QProxyStyle::pixelMetric(metric, option, widget);
    }

    void drawPrimitive(PrimitiveElement part, const QStyleOption *option, QPainter *painter,
                       const QWidget *widget) const override
    {
        if (part != PE_IndicatorCheckBox) {
            QProxyStyle::drawPrimitive(part, option, painter, widget);
            return;
        }
        const bool on = option->state.testFlag(State_On) || option->state.testFlag(State_NoChange);
        const bool enabled = option->state.testFlag(State_Enabled);
        const auto group = enabled ? QPalette::Active : QPalette::Disabled;
        const auto palette = option->palette;
        const QRectF rect = QRectF(option->rect).adjusted(1, 1, -1, -1);
        painter->save();
        painter->setRenderHint(QPainter::Antialiasing);
        painter->setPen(QPen(palette.color(group, on || option->state.testFlag(State_HasFocus)
                                                      ? QPalette::Highlight : QPalette::Mid), 1.2));
        painter->setBrush(palette.color(group, on ? QPalette::Highlight : QPalette::Base));
        painter->drawRoundedRect(rect, 3, 3);
        if (on) {
            const QPointF center = rect.center();
            painter->setPen(QPen(palette.color(group, QPalette::HighlightedText), 1.8,
                                 Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
            if (option->state.testFlag(State_NoChange))
                painter->drawLine(center + QPointF(-4, 0), center + QPointF(4, 0));
            else
                painter->drawPolyline(QPolygonF{center + QPointF(-4, 0), center + QPointF(-1, 3),
                                                center + QPointF(4, -3)});
        }
        painter->restore();
    }
};
}

QIcon uiIcon(UiIcon symbol, const QColor &color)
{
    QIcon icon;
    const auto ink = color.isValid() ? color : QApplication::palette().color(QPalette::WindowText);
    for (const int factor : {1, 2, 3}) {
        QPixmap pixmap(18 * factor, 18 * factor);
        pixmap.fill(Qt::transparent);
        QPainter p(&pixmap);
        p.scale(factor, factor);
        p.setRenderHint(QPainter::Antialiasing);
        p.setPen(QPen(ink, 1.5, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        switch (symbol) {
        case UiIcon::Add:
            p.drawLine(4, 9, 14, 9);
            p.drawLine(9, 4, 9, 14);
            break;
        case UiIcon::Automations:
            p.drawRoundedRect(QRectF(3, 4, 12, 11), 2, 2);
            p.drawLine(6, 2, 6, 6);
            p.drawLine(12, 2, 12, 6);
            p.drawLine(5, 9, 13, 9);
            break;
        case UiIcon::Collapse:
            p.drawPolyline(QPolygonF{{11, 4}, {6, 9}, {11, 14}});
            break;
        case UiIcon::Expand:
            p.drawPolyline(QPolygonF{{7, 4}, {12, 9}, {7, 14}});
            break;
        case UiIcon::Settings: {
            const QPointF center(9, 9);
            QPolygonF outline;
            for (int corner = 0; corner < 24; ++corner) {
                const auto angle = qDegreesToRadians(corner * 15.0 - 112.5);
                const bool tooth = corner % 4 == 1 || corner % 4 == 2;
                const auto radius = tooth ? 6.7 : 5.1;
                outline.append(center + QPointF(radius * qCos(angle), radius * qSin(angle)));
            }
            p.drawPolygon(outline);
            p.drawEllipse(center, 1.9, 1.9);
            break;
        }
        case UiIcon::Status:
            p.setBrush(ink);
            p.drawEllipse(QPointF(9, 9), 2.7, 2.7);
            break;
        case UiIcon::Close:
            p.drawLine(5, 5, 13, 13);
            p.drawLine(5, 13, 13, 5);
            break;
        case UiIcon::Success:
            p.drawPolyline(QPolygonF{{4, 9}, {7, 12}, {14, 5}});
            break;
        case UiIcon::Info:
            p.drawEllipse(QPointF(9, 9), 6.5, 6.5);
            p.drawPoint(9, 6);
            p.drawLine(9, 8.5, 9, 12);
            break;
        case UiIcon::Warning:
            p.drawPolygon(QPolygonF{{9, 2}, {16, 15}, {2, 15}});
            p.drawLine(9, 7, 9, 10);
            p.drawPoint(9, 12);
            break;
        }
        p.end();
        pixmap.setDevicePixelRatio(factor);
        icon.addPixmap(pixmap);
    }
    return icon;
}

void applyTheme(QApplication &app)
{
    Q_INIT_RESOURCE(cgacp_ui);
    systemFont = app.font();
    systemPalette = app.palette();
    app.setWindowIcon(QIcon(QStringLiteral(":/cgacp/favicon.png")));
    app.setStyle(new CheckStyle);
}

ThemeManager::ThemeManager(QObject *parent) : QObject(parent)
{
    connect(QGuiApplication::styleHints(), &QStyleHints::colorSchemeChanged, this, [this] {
        if (theme_ == QStringLiteral("system"))
            apply(theme_, density_, textScale_);
    });
}

void ThemeManager::apply(const QString &theme, const QString &density, int textScale)
{
    const auto scheme = QGuiApplication::styleHints()->colorScheme();
    const bool light = theme == "light" || (theme == "system" &&
                       (scheme == Qt::ColorScheme::Light ||
                        (scheme == Qt::ColorScheme::Unknown &&
                         systemPalette.color(QPalette::Window).lightness() >= 128)));
    if (theme_ == theme && density_ == density && textScale_ == textScale && light_ == light)
        return;
    theme_ = theme;
    density_ = density;
    textScale_ = textScale;
    light_ = light;
    const auto tone = [light](const char *dark, const char *bright) {
        return QString::fromLatin1(light ? bright : dark);
    };
    const auto canvas = tone("#181818", "#f6f6f5");
    const auto rail = tone("#1c1c1c", "#eeeeec");
    const auto surface = tone("#232323", "#ffffff");
    const auto base = tone("#202020", "#ffffff");
    const auto border = tone("#3b3b3b", "#cbcaca");
    const auto control = tone("#747474", "#777774");
    const auto text = tone("#ececec", "#202020");
    const auto muted = tone("#b1b1b1", "#575757");
    const auto hover = tone("#303030", "#e7e7e3");
    const auto selected = tone("#29423e", "#d6ebe7");
    const auto accent = tone("#70d7c6", "#006a5e");
    const auto onAccent = tone("#102421", "#ffffff");
    const auto danger = tone("#ffaaa3", "#9e2720");
    const auto warning = tone("#ebbf83", "#794908");

    QPalette palette;
    palette.setColor(QPalette::Window, QColor(canvas));
    palette.setColor(QPalette::WindowText, QColor(text));
    palette.setColor(QPalette::Base, QColor(base));
    palette.setColor(QPalette::AlternateBase, QColor(surface));
    palette.setColor(QPalette::Text, QColor(text));
    palette.setColor(QPalette::Button, QColor(surface));
    palette.setColor(QPalette::ButtonText, QColor(text));
    palette.setColor(QPalette::ToolTipBase, QColor(surface));
    palette.setColor(QPalette::ToolTipText, QColor(text));
    palette.setColor(QPalette::PlaceholderText, QColor(muted));
    palette.setColor(QPalette::Mid, QColor(control));
    palette.setColor(QPalette::Highlight, QColor(accent));
    palette.setColor(QPalette::HighlightedText, QColor(onAccent));
    palette.setColor(QPalette::Link, QColor(accent));
    for (const auto role : {QPalette::Text, QPalette::WindowText, QPalette::ButtonText})
        palette.setColor(QPalette::Disabled, role, QColor(muted));
    palette.setColor(QPalette::Disabled, QPalette::Highlight, QColor(control));
    palette.setColor(QPalette::Disabled, QPalette::HighlightedText, QColor(onAccent));
    QApplication::setPalette(palette);
    QFont font = systemFont;
    if (font.pointSizeF() > 0)
        font.setPointSizeF(font.pointSizeF() * textScale / 100.0);
    else if (font.pixelSize() > 0)
        font.setPixelSize(qRound(font.pixelSize() * textScale / 100.0));
    QApplication::setFont(font);

    QFile file(":/cgacp/theme.qss");
    if (file.open(QIODevice::ReadOnly)) {
        auto style = QString::fromUtf8(file.readAll());
        const auto scaled = [textScale](int size) { return QString::number(qRound(size * textScale / 100.0)); };
        const QList<QPair<QString, QString>> tokens{
            {"CANVAS", canvas}, {"RAIL", rail}, {"SURFACE", surface}, {"BASE", base},
            {"BORDER", border}, {"CONTROL", control}, {"TEXT", text}, {"MUTED", muted},
            {"HOVER", hover}, {"SELECTED", selected}, {"ACCENT", accent}, {"ON_ACCENT", onAccent},
            {"DANGER", danger}, {"WARNING", warning}, {"BODY_SIZE", scaled(13)}, {"TITLE_SIZE", scaled(24)},
            {"SECTION_SIZE", scaled(15)}, {"META_SIZE", scaled(12)},
            {"VERTICAL_PAD", density == "compact" ? "4" : "6"}
        };
        for (const auto &[token, value] : tokens)
            style.replace("@" + token + "@", value);
        qApp->setStyleSheet(style);
    }
    emit changed();
}
