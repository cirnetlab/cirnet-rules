#include "theme.h"

#include <QAbstractItemView>
#include <QApplication>
#include <QComboBox>
#include <QDialogButtonBox>
#include <QFile>
#include <QFocusEvent>
#include <QIconEngine>
#include <QPainter>
#include <QPainterPath>
#include <QPixmapCache>
#include <QPointer>
#include <QProxyStyle>
#include <QPushButton>
#include <QStyleFactory>
#include <QStyleHints>
#include <QStyleOption>
#include <QToolButton>
#include <QtMath>

namespace {
QFont systemFont;
QPalette systemPalette;
constexpr auto keyboardFocusProperty = "keyboardFocusVisible";
constexpr int stepperWidth = 28;

class InteractionState final : public QObject {
public:
    explicit InteractionState(QApplication &app) : QObject(&app)
    {
        app.installEventFilter(this);
    }

protected:
    bool eventFilter(QObject *receiver, QEvent *event) override
    {
        switch (event->type()) {
        case QEvent::MouseButtonPress:
        case QEvent::TouchBegin:
            keyboard_ = false;
            setFocusVisible(nullptr);
            break;
        case QEvent::KeyPress:
            keyboard_ = true;
            setFocusVisible(QApplication::focusWidget());
            break;
        case QEvent::FocusIn: {
            // Qt also dispatches the focus event to its styles after the widget.
            auto *widget = qobject_cast<QWidget *>(receiver);
            if (!widget || widget != QApplication::focusWidget())
                break;
            const auto reason = static_cast<QFocusEvent *>(event)->reason();
            if (reason == Qt::TabFocusReason || reason == Qt::BacktabFocusReason || reason == Qt::ShortcutFocusReason)
                keyboard_ = true;
            else if (reason == Qt::MouseFocusReason)
                keyboard_ = false;
            setFocusVisible(keyboard_ ? widget : nullptr);
            break;
        }
        case QEvent::FocusOut:
            if (receiver == focus_)
                setFocusVisible(nullptr);
            break;
        case QEvent::Polish:
            if (auto *button = qobject_cast<QPushButton *>(receiver)) {
                const auto *box = qobject_cast<QDialogButtonBox *>(button->parentWidget());
                if (box && box->standardButton(button) == QDialogButtonBox::Save)
                    button->setProperty("primary", true);
            }
            break;
        case QEvent::Wheel:
            if (auto *widget = qobject_cast<QWidget *>(receiver)) {
                for (auto *candidate : {widget, widget->parentWidget()}) {
                    if (candidate && (qobject_cast<QAbstractSpinBox *>(candidate) || qobject_cast<QComboBox *>(candidate)) &&
                        !candidate->hasFocus()) {
                        event->ignore();
                        return true;
                    }
                }
            }
            break;
        default:
            break;
        }
        return QObject::eventFilter(receiver, event);
    }

private:
    void setFocusVisible(QWidget *widget)
    {
        if (!qobject_cast<QAbstractButton *>(widget) && !qobject_cast<QAbstractItemView *>(widget))
            widget = nullptr;
        if (focus_ == widget)
            return;
        const auto previous = focus_;
        focus_ = widget;
        for (const auto &control : {previous, focus_}) {
            if (!control)
                continue;
            control->setProperty(keyboardFocusProperty, control == focus_);
            control->style()->unpolish(control);
            control->style()->polish(control);
            control->update();
        }
    }

    QPointer<QWidget> focus_;
    bool keyboard_ = false;
};

void drawChevron(QPainter &painter, const QRect &rect, Qt::ArrowType direction, const QColor &color)
{
    if (rect.width() < 4 || rect.height() < 4)
        return;
    const qreal halfWidth = qMin<qreal>(4.0, (rect.width() - 1) / 2.0);
    const qreal halfHeight = qMin<qreal>(2.3, (rect.height() - 1) / 2.0);
    painter.save();
    painter.setRenderHint(QPainter::Antialiasing);
    painter.translate(QRectF(rect).center());
    switch (direction) {
    case Qt::UpArrow: painter.rotate(180); break;
    case Qt::LeftArrow: painter.rotate(90); break;
    case Qt::RightArrow: painter.rotate(-90); break;
    default: break;
    }
    painter.setPen(QPen(color, 1.6, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
    painter.drawPolyline(QPolygonF{{-halfWidth, -halfHeight}, {0, halfHeight}, {halfWidth, -halfHeight}});
    painter.restore();
}

QRect stepperRect(const QStyleOptionComplex &option)
{
    const auto interior = option.rect.adjusted(1, 1, -1, -1);
    const auto width = qBound(0, interior.width(), stepperWidth);
    return {interior.right() - width + 1, interior.top(), width, interior.height()};
}

void drawFieldFrame(const QStyleOptionComplex &option, QPainter &painter)
{
    const auto enabled = option.state.testFlag(QStyle::State_Enabled);
    const auto &palette = option.palette;
    painter.setPen(option.state.testFlag(QStyle::State_HasFocus) ? palette.highlight().color()
                                                                : palette.dark().color());
    painter.setBrush(enabled ? palette.base() : palette.window());
    painter.drawRoundedRect(QRectF(option.rect).adjusted(0.5, 0.5, -0.5, -0.5), 6, 6);
}

class DashboardStyle final : public QProxyStyle {
public:
    DashboardStyle() : QProxyStyle(QStyleFactory::create("Fusion")) {}

    void polish(QWidget *widget) override
    {
        QProxyStyle::polish(widget);
        if (qobject_cast<QAbstractSpinBox *>(widget) || qobject_cast<QComboBox *>(widget))
            widget->setAttribute(Qt::WA_Hover);
    }

    int pixelMetric(PixelMetric metric, const QStyleOption *option, const QWidget *widget) const override
    {
        if (metric == PM_IndicatorWidth || metric == PM_IndicatorHeight ||
            metric == PM_ExclusiveIndicatorWidth || metric == PM_ExclusiveIndicatorHeight)
            return 17;
        return QProxyStyle::pixelMetric(metric, option, widget);
    }

    QSize sizeFromContents(ContentsType type, const QStyleOption *option, const QSize &size,
                           const QWidget *widget) const override
    {
        const auto native = QProxyStyle::sizeFromContents(type, option, size, widget);
        if (type == CT_ComboBox || type == CT_SpinBox)
            return native.expandedTo(size + QSize(type == CT_ComboBox ? stepperWidth + 16 : 16, fieldPadding_));
        return native;
    }

    void setCompact(bool compact)
    {
        fieldPadding_ = compact ? 10 : 14;
    }

    QRect subControlRect(ComplexControl type, const QStyleOptionComplex *option, SubControl part,
                         const QWidget *widget) const override
    {
        const auto stepper = stepperRect(*option);
        const auto mirrored = [option](const QRect &rect) {
            return visualRect(option->direction, option->rect, rect);
        };
        if (type == CC_ComboBox) {
            if (part == SC_ComboBoxArrow)
                return mirrored(stepper);
            if (part == SC_ComboBoxEditField)
                return mirrored(option->rect.adjusted(8, 1, -stepper.width() - 5, -1));
        } else if (type == CC_SpinBox) {
            const auto *spin = qstyleoption_cast<const QStyleOptionSpinBox *>(option);
            const bool buttons = spin && spin->buttonSymbols != QAbstractSpinBox::NoButtons;
            if (part == SC_SpinBoxUp || part == SC_SpinBoxDown) {
                if (!buttons)
                    return {};
                const int midpoint = stepper.top() + (stepper.height() + 1) / 2;
                return mirrored(part == SC_SpinBoxUp
                    ? QRect(stepper.topLeft(), QPoint(stepper.right(), midpoint - 1))
                    : QRect(QPoint(stepper.left(), midpoint), stepper.bottomRight()));
            }
            if (part == SC_SpinBoxEditField)
                return mirrored(option->rect.adjusted(8, 1, -(buttons ? stepper.width() : 0) - 5, -1));
        }
        return QProxyStyle::subControlRect(type, option, part, widget);
    }

    void drawComplexControl(ComplexControl type, const QStyleOptionComplex *option, QPainter *painter,
                            const QWidget *widget) const override
    {
        if (type != CC_ComboBox && type != CC_SpinBox) {
            QProxyStyle::drawComplexControl(type, option, painter, widget);
            return;
        }
        painter->save();
        painter->setRenderHint(QPainter::Antialiasing);
        if ((type == CC_ComboBox && option->subControls.testFlag(SC_ComboBoxFrame)) ||
            (type == CC_SpinBox && option->subControls.testFlag(SC_SpinBoxFrame)))
            drawFieldFrame(*option, *painter);

        const auto arrow = [&](SubControl control, Qt::ArrowType direction, bool stepEnabled) {
            if (!option->subControls.testFlag(control))
                return;
            const auto bounds = subControlRect(type, option, control, widget);
            const bool enabled = option->state.testFlag(State_Enabled) && stepEnabled;
            if (enabled && option->activeSubControls.testFlag(control) &&
                (option->state.testFlag(State_MouseOver) || option->state.testFlag(State_Sunken))) {
                painter->setPen(Qt::NoPen);
                painter->setBrush(option->palette.midlight());
                painter->drawRoundedRect(QRectF(bounds).adjusted(2, 1, -2, -1), 4, 4);
            }
            drawChevron(*painter, bounds, direction,
                        option->palette.color(enabled ? QPalette::Active : QPalette::Disabled, QPalette::ButtonText));
        };
        if (type == CC_ComboBox) {
            arrow(SC_ComboBoxArrow, Qt::DownArrow, true);
        } else if (const auto *spin = qstyleoption_cast<const QStyleOptionSpinBox *>(option)) {
            if (spin->buttonSymbols == QAbstractSpinBox::PlusMinus) {
                for (const auto part : {SC_SpinBoxUp, SC_SpinBoxDown}) {
                    QStyleOption marker = *spin;
                    marker.rect = subControlRect(type, option, part, widget);
                    if (!spin->stepEnabled.testFlag(part == SC_SpinBoxUp ? QAbstractSpinBox::StepUpEnabled
                                                                       : QAbstractSpinBox::StepDownEnabled))
                        marker.state &= ~State_Enabled;
                    QProxyStyle::drawPrimitive(part == SC_SpinBoxUp ? PE_IndicatorSpinPlus : PE_IndicatorSpinMinus,
                                               &marker, painter, widget);
                }
            } else if (spin->buttonSymbols == QAbstractSpinBox::UpDownArrows) {
                arrow(SC_SpinBoxUp, Qt::UpArrow, spin->stepEnabled.testFlag(QAbstractSpinBox::StepUpEnabled));
                arrow(SC_SpinBoxDown, Qt::DownArrow, spin->stepEnabled.testFlag(QAbstractSpinBox::StepDownEnabled));
            }
        }
        painter->restore();
    }

    void drawPrimitive(PrimitiveElement part, const QStyleOption *option, QPainter *painter,
                       const QWidget *widget) const override
    {
        if (part == PE_FrameFocusRect &&
            (qobject_cast<const QAbstractButton *>(widget) ||
             (qobject_cast<const QAbstractItemView *>(widget) && !widget->property(keyboardFocusProperty).toBool())))
            return;

        Qt::ArrowType direction = Qt::NoArrow;
        switch (part) {
        case PE_IndicatorArrowDown:
        case PE_IndicatorSpinDown: direction = Qt::DownArrow; break;
        case PE_IndicatorArrowUp:
        case PE_IndicatorSpinUp: direction = Qt::UpArrow; break;
        case PE_IndicatorArrowLeft: direction = Qt::LeftArrow; break;
        case PE_IndicatorArrowRight: direction = Qt::RightArrow; break;
        case PE_IndicatorHeaderArrow:
            if (const auto *header = qstyleoption_cast<const QStyleOptionHeader *>(option))
                direction = header->sortIndicator == QStyleOptionHeader::SortUp ? Qt::UpArrow : Qt::DownArrow;
            break;
        default:
            break;
        }
        if (direction != Qt::NoArrow) {
            auto rect = option->rect;
            const auto *push = qobject_cast<const QPushButton *>(widget);
            const auto *tool = qobject_cast<const QToolButton *>(widget);
            if (part == PE_IndicatorArrowDown && ((push && push->menu()) ||
                (tool && tool->menu() && tool->popupMode() == QToolButton::InstantPopup))) {
                rect = visualRect(widget->layoutDirection(), widget->rect(),
                                  QRect(widget->width() - 21, 0, 16, widget->height()));
            }
            drawChevron(*painter, rect, direction, option->palette.color(
                option->state.testFlag(State_Enabled) ? QPalette::Active : QPalette::Disabled, QPalette::ButtonText));
            return;
        }
        if (part != PE_IndicatorCheckBox && part != PE_IndicatorRadioButton) {
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
        painter->setPen(QPen(palette.color(group, on
                                                      ? QPalette::Highlight : QPalette::Mid), 1.2));
        if (part == PE_IndicatorRadioButton) {
            painter->setBrush(palette.color(group, QPalette::Base));
            painter->drawEllipse(rect);
            if (on) {
                painter->setPen(Qt::NoPen);
                painter->setBrush(palette.color(group, QPalette::Highlight));
                painter->drawEllipse(rect.adjusted(4, 4, -4, -4));
            }
            painter->restore();
            return;
        }
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

    QIcon standardIcon(StandardPixmap icon, const QStyleOption *option, const QWidget *widget) const override
    {
        switch (icon) {
        case SP_LineEditClearButton: return uiIcon(UiIcon::Clear);
        case SP_BrowserReload: return uiIcon(UiIcon::Refresh);
        case SP_MessageBoxInformation: return uiIcon(UiIcon::Info);
        case SP_MessageBoxWarning: return uiIcon(UiIcon::Warning);
        case SP_MessageBoxCritical: return uiIcon(UiIcon::Error);
        case SP_MessageBoxQuestion: return uiIcon(UiIcon::Question);
        default: return QProxyStyle::standardIcon(icon, option, widget);
        }
    }

private:
    int fieldPadding_ = 14;
};

QPointer<DashboardStyle> dashboardStyle;

class VectorIcon final : public QIconEngine {
public:
    VectorIcon(UiIcon symbol, QColor color) : symbol_(symbol), color_(color) {}

    QIconEngine *clone() const override { return new VectorIcon(symbol_, color_); }

    QPixmap pixmap(const QSize &size, QIcon::Mode mode, QIcon::State state) override
    {
        if (size.isEmpty())
            return {};
        const auto ink = iconColor(mode);
        const auto key = QStringLiteral("cgacp-vector-%1-%2-%3-%4")
                             .arg(static_cast<int>(symbol_)).arg(ink.rgba())
                             .arg(size.width()).arg(size.height());
        QPixmap result;
        if (QPixmapCache::find(key, &result))
            return result;
        result = QPixmap(size);
        result.fill(Qt::transparent);
        {
            QPainter painter(&result);
            paint(&painter, result.rect(), mode, state);
        }
        QPixmapCache::insert(key, result);
        return result;
    }

    QPixmap scaledPixmap(const QSize &size, QIcon::Mode mode, QIcon::State state, qreal scale) override
    {
#if QT_VERSION >= QT_VERSION_CHECK(6, 8, 0)
        return pixmap(size * scale, mode, state);
#else
        Q_UNUSED(scale);
        return pixmap(size, mode, state);
#endif
    }

    void paint(QPainter *painter, const QRect &rect, QIcon::Mode mode, QIcon::State) override
    {
        const auto extent = qMin(rect.width(), rect.height());
        if (extent <= 0)
            return;
        auto &p = *painter;
        const auto ink = iconColor(mode);
        p.save();
        p.translate(QRectF(rect).center() - QPointF(extent / 2.0, extent / 2.0));
        p.scale(extent / 18.0, extent / 18.0);
        p.setRenderHint(QPainter::Antialiasing);
        p.setPen(QPen(ink, 1.55, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
        p.setBrush(Qt::NoBrush);
        switch (symbol_) {
        case UiIcon::Add:
            p.drawLine(4, 9, 14, 9);
            p.drawLine(9, 4, 9, 14);
            break;
        case UiIcon::Automations:
            p.drawRoundedRect(QRectF(3, 4.5, 12, 11), 2, 2);
            p.drawLine(QPointF(6, 2.5), QPointF(6, 5.7));
            p.drawLine(QPointF(12, 2.5), QPointF(12, 5.7));
            p.drawLine(QPointF(3, 8), QPointF(15, 8));
            p.drawPoint(9, 11);
            break;
        case UiIcon::Collapse:
        case UiIcon::Expand: {
            p.drawRoundedRect(QRectF(2, 3, 14, 12), 2.2, 2.2);
            p.drawLine(QPointF(6.5, 3), QPointF(6.5, 15));
            const bool collapse = symbol_ == UiIcon::Collapse;
            const qreal tail = collapse ? 12.7 : 9.8;
            const qreal tip = collapse ? 10.3 : 12.2;
            p.drawPolyline(QPolygonF{{tail, 6.9}, {tip, 9}, {tail, 11.1}});
            break;
        }
        case UiIcon::Settings: {
            const QPointF center(9, 9);
            QPolygonF outline;
            for (int corner = 0; corner < 24; ++corner) {
                const auto angle = qDegreesToRadians(corner * 15.0 - 112.5);
                const bool tooth = corner % 4 == 1 || corner % 4 == 2;
                const auto radius = tooth ? 6.7 : 5.25;
                outline.append(center + QPointF(radius * qCos(angle), radius * qSin(angle)));
            }
            p.drawPolygon(outline);
            p.drawEllipse(center, 1.9, 1.9);
            break;
        }
        case UiIcon::Status:
            p.setPen(Qt::NoPen);
            p.setBrush(ink);
            p.drawEllipse(QPointF(9, 9), 3.3, 3.3);
            break;
        case UiIcon::Close:
        case UiIcon::Clear:
            p.drawLine(5, 5, 13, 13);
            p.drawLine(5, 13, 13, 5);
            break;
        case UiIcon::Refresh:
            p.drawArc(QRectF(3, 3, 12, 12), 34 * 16, 300 * 16);
            p.drawPolyline(QPolygonF{{14.5, 2.6}, {14.1, 5.9}, {10.8, 5.8}});
            break;
        case UiIcon::Success:
            p.drawEllipse(QPointF(9, 9), 6.5, 6.5);
            p.drawPolyline(QPolygonF{{5.8, 9}, {8.0, 11.2}, {12.2, 7}});
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
        case UiIcon::Error:
            p.drawEllipse(QPointF(9, 9), 6.5, 6.5);
            p.drawLine(QPointF(6.8, 6.8), QPointF(11.2, 11.2));
            p.drawLine(QPointF(6.8, 11.2), QPointF(11.2, 6.8));
            break;
        case UiIcon::Question: {
            p.drawEllipse(QPointF(9, 9), 6.5, 6.5);
            QPainterPath hook(QPointF(6.8, 6.6));
            hook.cubicTo(7.1, 4.7, 11.2, 4.6, 11.2, 6.8);
            hook.cubicTo(11.2, 8.1, 9, 8.4, 9, 9.9);
            p.drawPath(hook);
            p.drawPoint(QPointF(9, 12.2));
            break;
        }
        }
        p.restore();
    }

private:
    QColor iconColor(QIcon::Mode mode) const
    {
        const auto palette = QApplication::palette();
        if (mode == QIcon::Disabled)
            return palette.color(QPalette::Disabled, QPalette::ButtonText);
        if (color_.isValid())
            return color_;
        const bool light = palette.color(QPalette::Window).lightness() >= 128;
        switch (symbol_) {
        case UiIcon::Clear: return palette.color(QPalette::PlaceholderText);
        case UiIcon::Info:
        case UiIcon::Question: return QColor(light ? "#245f94" : "#8bbcf0");
        case UiIcon::Success: return QColor(light ? "#176546" : "#80d5a6");
        case UiIcon::Warning: return QColor(light ? "#794908" : "#ebbf83");
        case UiIcon::Error: return QColor(light ? "#9e2720" : "#ffaaa3");
        default: return palette.color(QPalette::WindowText);
        }
    }

    UiIcon symbol_;
    QColor color_;
};
}

QIcon uiIcon(UiIcon symbol, const QColor &color)
{
    return QIcon(new VectorIcon(symbol, color));
}

void applyTheme(QApplication &app)
{
    Q_INIT_RESOURCE(cgacp_ui);
    systemFont = app.font();
    systemPalette = app.palette();
    app.setWindowIcon(QIcon(QStringLiteral(":/cgacp/favicon.png")));
    dashboardStyle = new DashboardStyle;
    app.setStyle(dashboardStyle);
    new InteractionState(app);
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
    if (dashboardStyle)
        dashboardStyle->setCompact(density == "compact");
    const auto tone = [light](const char *dark, const char *bright) {
        return QString::fromLatin1(light ? bright : dark);
    };
    const auto canvas = tone("#181818", "#f6f6f5");
    const auto rail = tone("#1c1c1c", "#eeeeec");
    const auto surface = tone("#232323", "#ffffff");
    const auto base = tone("#202020", "#ffffff");
    const auto alternateRow = tone("#232323", "#f5f5f3");
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
    palette.setColor(QPalette::AlternateBase, QColor(alternateRow));
    palette.setColor(QPalette::Text, QColor(text));
    palette.setColor(QPalette::Button, QColor(surface));
    palette.setColor(QPalette::ButtonText, QColor(text));
    palette.setColor(QPalette::ToolTipBase, QColor(surface));
    palette.setColor(QPalette::ToolTipText, QColor(text));
    palette.setColor(QPalette::PlaceholderText, QColor(muted));
    palette.setColor(QPalette::Mid, QColor(control));
    palette.setColor(QPalette::Dark, QColor(border));
    palette.setColor(QPalette::Midlight, QColor(hover));
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
            {"CANVAS", canvas}, {"RAIL", rail}, {"SURFACE", surface}, {"BASE", base}, {"ALTERNATE_ROW", alternateRow},
            {"BORDER", border}, {"CONTROL", control}, {"TEXT", text}, {"MUTED", muted},
            {"HOVER", hover}, {"SELECTED", selected}, {"ACCENT", accent}, {"ON_ACCENT", onAccent},
            {"PRIMARY_HOVER", tone("#92e6d8", "#08786a")}, {"PRIMARY_PRESSED", tone("#58c7b5", "#00594f")},
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
