#include "build_info.h"
#include "theme.h"

#include <QAbstractItemView>
#include <QAbstractScrollArea>
#include <QAbstractSlider>
#include <QAction>
#include <QApplication>
#include <QFontDatabase>
#include <QComboBox>
#include <QDialogButtonBox>
#include <QFile>
#include <QFocusEvent>
#include <QIconEngine>
#include <QLineEdit>
#include <QPainter>
#include <QPainterPath>
#include <QPixmapCache>
#include <QPointer>
#include <QProxyStyle>
#include <QPushButton>
#include <QScrollBar>
#include <QStyleFactory>
#include <QStyleHints>
#include <QStyleOption>
#include <QTimer>
#include <QToolButton>
#include <QWheelEvent>
#include <QtMath>
#include <cmath>

namespace {
QFont systemFont;
QPalette systemPalette;
constexpr auto keyboardFocusProperty = "keyboardFocusVisible";
constexpr auto scrollRevealedProperty = "scrollRevealed";
constexpr auto scrollHoverProperty = "scrollPointerOver";
constexpr auto wheelRevealTimer = "scrollWheelRevealTimer";
constexpr int stepperWidth = 28;

QColor hexColor(const QString &text)
{
    return text.size() == 7 && text.startsWith(u'#') ? QColor(text) : QColor{};
}

QColor blend(const QColor &base, const QColor &over, double amount)
{
    const auto channel = [amount](int from, int to) { return qRound(from + (to - from) * amount); };
    return QColor(channel(base.red(), over.red()), channel(base.green(), over.green()),
                  channel(base.blue(), over.blue()));
}

double luminance(const QColor &color)
{
    const auto linear = [](double channel) {
        return channel <= 0.04045 ? channel / 12.92 : std::pow((channel + 0.055) / 1.055, 2.4);
    };
    return 0.2126 * linear(color.redF()) + 0.7152 * linear(color.greenF()) + 0.0722 * linear(color.blueF());
}

void revealScrollBar(QScrollBar *bar, bool visible)
{
    if (!bar || bar->property(scrollRevealedProperty).toBool() == visible)
        return;
    bar->setProperty(scrollRevealedProperty, visible);
    bar->style()->unpolish(bar);
    bar->style()->polish(bar);
    bar->update();
}

void flashScrollBar(QScrollBar *bar)
{
    if (!bar->isVisible() || bar->minimum() == bar->maximum())
        return;
    auto *timer = bar->findChild<QTimer *>(wheelRevealTimer, Qt::FindDirectChildrenOnly);
    if (!timer) {
        timer = new QTimer(bar);
        timer->setObjectName(wheelRevealTimer);
        timer->setSingleShot(true);
        QObject::connect(timer, &QTimer::timeout, bar, [bar] {
            revealScrollBar(bar, bar->property(scrollHoverProperty).toBool());
        });
    }
    revealScrollBar(bar, true);
    timer->start(850);
}

void revealWheelScrollBars(QWidget *widget, const QWheelEvent &event)
{
    const auto delta = event.pixelDelta().isNull() ? event.angleDelta() : event.pixelDelta();
    if (delta.isNull())
        return;
    if (auto *bar = qobject_cast<QScrollBar *>(widget)) {
        flashScrollBar(bar);
        return;
    }
    for (auto *ancestor = widget; ancestor; ancestor = ancestor->parentWidget()) {
        if (auto *area = qobject_cast<QAbstractScrollArea *>(ancestor)) {
            if (delta.y())
                flashScrollBar(area->verticalScrollBar());
            if (delta.x())
                flashScrollBar(area->horizontalScrollBar());
            break;
        }
        if (ancestor->isWindow())
            break;
    }
}

bool routeEditorWheel(QWidget *widget, QWheelEvent &event)
{
    bool overEditor = false;
    for (auto *ancestor = widget; ancestor; ancestor = ancestor->parentWidget()) {
        if (auto *area = qobject_cast<QAbstractScrollArea *>(ancestor)) {
            if (!overEditor)
                return false;
            auto *viewport = area->viewport();
            QWheelEvent forwarded(viewport->mapFromGlobal(event.globalPosition()), event.globalPosition(),
                                  event.pixelDelta(), event.angleDelta(), event.buttons(), event.modifiers(),
                                  event.phase(), event.inverted(), event.source(), event.pointingDevice());
            forwarded.ignore();
            QApplication::sendEvent(viewport, &forwarded);
            if (forwarded.isAccepted())
                break;
        }
        overEditor = overEditor || qobject_cast<QAbstractSpinBox *>(ancestor)
            || qobject_cast<QComboBox *>(ancestor)
            || (qobject_cast<QAbstractSlider *>(ancestor) && !qobject_cast<QScrollBar *>(ancestor));
        if (ancestor->isWindow())
            break;
    }
    if (overEditor)
        event.accept();
    return overEditor;
}

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
            if (auto *field = qobject_cast<QLineEdit *>(receiver);
                field && field->property("searchField").toBool() && !field->findChild<QAction *>("focusSearchField")) {
                auto *search = field->addAction(uiIcon(UiIcon::Search), QLineEdit::LeadingPosition);
                search->setObjectName("focusSearchField");
                connect(search, &QAction::triggered, field, [field] { field->setFocus(Qt::MouseFocusReason); });
            }
            break;
        case QEvent::Enter:
        case QEvent::Leave:
            if (auto *bar = qobject_cast<QScrollBar *>(receiver)) {
                const bool reveal = event->type() == QEvent::Enter;
                bar->setProperty(scrollHoverProperty, reveal);
                const auto *timer = bar->findChild<QTimer *>(wheelRevealTimer, Qt::FindDirectChildrenOnly);
                revealScrollBar(bar, reveal || (timer && timer->isActive()));
            }
            break;
        case QEvent::Wheel:
            if (auto *widget = qobject_cast<QWidget *>(receiver)) {
                auto &wheel = *static_cast<QWheelEvent *>(event);
                if (routeEditorWheel(widget, wheel))
                    return true;
                revealWheelScrollBars(widget, wheel);
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
    painter.setPen(option.state.testFlag(QStyle::State_HasFocus) ? palette.link().color()
                                                                : palette.dark().color());
    painter.setBrush(enabled ? palette.base() : palette.window());
    painter.drawRoundedRect(QRectF(option.rect).adjusted(0.5, 0.5, -0.5, -0.5), 9, 9);
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

    void setVerticalPadding(int pixels)
    {
        fieldPadding_ = 2 * pixels + 2;
    }

    int styleHint(StyleHint hint, const QStyleOption *option, const QWidget *widget,
                  QStyleHintReturn *data) const override
    {
        if (hint == SH_UnderlineShortcut)
            return 0;
        if (hint == SH_ComboBox_Popup && widget && widget->objectName() == "uiFontChoice")
            return 0;
        return QProxyStyle::styleHint(hint, option, widget, data);
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
        const auto key = QStringLiteral("cirnetrules-vector-%1-%2-%3-%4")
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
        case UiIcon::Search:
            p.drawEllipse(QPointF(7.6, 7.6), 4.6, 4.6);
            p.drawLine(QPointF(11.1, 11.1), QPointF(15, 15));
            break;
        case UiIcon::Back:
            p.drawPolyline(QPolygonF{{8, 4}, {3, 9}, {8, 14}});
            p.drawLine(3, 9, 15, 9);
            break;
        case UiIcon::Appearance:
            p.drawRoundedRect(QRectF(2.5, 3.5, 13, 9.5), 1.8, 1.8);
            p.drawLine(9, 13, 9, 15);
            p.drawLine(6, 15, 12, 15);
            p.drawLine(5, 10, 11, 5);
            break;
        case UiIcon::Folder:
            p.drawPolygon(QPolygonF{{2.5, 5}, {7, 5}, {9, 7}, {15.5, 7}, {15.5, 14.5}, {2.5, 14.5}});
            break;
        case UiIcon::Sliders:
            p.drawLine(3, 6, 15, 6);
            p.drawLine(3, 12, 15, 12);
            p.setBrush(QApplication::palette().window());
            p.drawEllipse(QPointF(7, 6), 1.8, 1.8);
            p.drawEllipse(QPointF(11, 12), 1.8, 1.8);
            break;
        case UiIcon::Storage:
            p.drawRoundedRect(QRectF(2.5, 5, 13, 9), 2, 2);
            p.drawLine(3, 10, 15, 10);
            p.drawPoint(12, 12);
            break;
        case UiIcon::Archive:
            p.drawRoundedRect(QRectF(2.5, 3.5, 13, 4), 1.1, 1.1);
            p.drawPolyline(QPolygonF{{3.5, 7.5}, {3.5, 14.5}, {14.5, 14.5}, {14.5, 7.5}});
            p.drawLine(7, 10.5, 11, 10.5);
            break;
        case UiIcon::Runtime:
            p.drawRoundedRect(QRectF(2.5, 4, 13, 10), 1.8, 1.8);
            p.drawPolyline(QPolygonF{{5.2, 6.7}, {7.5, 9}, {5.2, 11.3}});
            p.drawLine(9.5, 11, 12.5, 11);
            break;
        case UiIcon::Bell: {
            QPainterPath bell(QPointF(3.5, 12.8));
            bell.cubicTo(5, 11, 5.3, 10.5, 5.3, 7.7);
            bell.cubicTo(5.3, 2.3, 12.7, 2.3, 12.7, 7.7);
            bell.cubicTo(12.7, 10.5, 13, 11, 14.5, 12.8);
            bell.closeSubpath();
            p.drawPath(bell);
            p.drawLine(8, 15.3, 10, 15.3);
            break;
        }
        case UiIcon::Trash:
            p.drawLine(3.5, 5.5, 14.5, 5.5);
            p.drawPolyline(QPolygonF{{5, 5.5}, {6, 15}, {12, 15}, {13, 5.5}});
            p.drawPolyline(QPolygonF{{7, 5.5}, {7, 3}, {11, 3}, {11, 5.5}});
            p.drawLine(8, 8, 8, 12);
            p.drawLine(10, 8, 10, 12);
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
        case UiIcon::Clear:
        case UiIcon::Search: return palette.color(QPalette::PlaceholderText);
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

QColor appearanceAccent(const QString &accent, bool light, const QString &custom)
{
    struct Preset { QLatin1StringView name; const char *dark; const char *light; };
    static constexpr Preset colors[]{
        {QLatin1StringView("cobalt"), "#94beff", "#174eaa"},
        {QLatin1StringView("green"), "#9cda71", "#376b22"},
        {QLatin1StringView("yellow"), "#f5ca60", "#856200"},
        {QLatin1StringView("pink"), "#f38ab6", "#a32a62"},
        {QLatin1StringView("orange"), "#ff9e72", "#9c471b"},
        {QLatin1StringView("purple"), "#c6a2ff", "#683ca5"},
        {QLatin1StringView("white"), "#ffffff", "#ffffff"}
    };
    if (accent == "custom") {
        const auto value = hexColor(custom);
        if (value.isValid())
            return value;
    }
    for (const auto &preset : colors)
        if (accent == preset.name)
            return QColor(light ? preset.light : preset.dark);
    return QColor(light ? "#17674a" : "#7cdbb2");
}

double appearanceColorContrast(const QColor &first, const QColor &second)
{
    const auto a = luminance(first);
    const auto b = luminance(second);
    return (qMax(a, b) + 0.05) / (qMin(a, b) + 0.05);
}

void applyTheme(QApplication &app)
{
    #define INIT_BUNDLE(name) Q_INIT_RESOURCE(name)
    INIT_BUNDLE(CIRNETRULES_RESOURCE_BUNDLE);
#undef INIT_BUNDLE
    systemFont = app.font();
    systemPalette = app.palette();
    app.setWindowIcon(QIcon(QStringLiteral(CIRNETRULES_RESOURCE_ROOT "/favicon.png")));
    dashboardStyle = new DashboardStyle;
    app.setStyle(dashboardStyle);
    new InteractionState(app);
}

ThemeManager::ThemeManager(QObject *parent) : QObject(parent)
{
    connect(QGuiApplication::styleHints(), &QStyleHints::colorSchemeChanged, this, [this] {
        if (theme_ == QStringLiteral("system"))
            apply(theme_, density_, textScale_, appearance_);
    });
}

void ThemeManager::apply(const QString &theme, const QString &density, int textScale,
                         const QJsonObject &appearance)
{
    const auto scheme = QGuiApplication::styleHints()->colorScheme();
    const bool light = theme == "light" || (theme == "system" &&
                       (scheme == Qt::ColorScheme::Light ||
                        (scheme == Qt::ColorScheme::Unknown &&
                         systemPalette.color(QPalette::Window).lightness() >= 128)));
    if (theme_ == theme && density_ == density && textScale_ == textScale &&
        light_ == light && appearance_ == appearance)
        return;
    theme_ = theme;
    density_ = density;
    textScale_ = textScale;
    appearance_ = appearance;
    light_ = light;
    const int padding = qBound(2, appearance.value("density_px").toInt(density == "compact" ? 4 : 6), 14);
    if (dashboardStyle)
        dashboardStyle->setVerticalPadding(padding);
    const auto customBackground = hexColor(appearance.value("background").toString());
    const auto customForeground = hexColor(appearance.value("foreground").toString());
    const auto canvas = customBackground.isValid() ? customBackground : QColor(light ? "#f5f8f6" : "#161918");
    const bool lightSurface = customBackground.isValid()
        ? appearanceColorContrast(canvas, Qt::black) >= appearanceColorContrast(canvas, Qt::white)
        : light;
    const auto tone = [lightSurface](const char *dark, const char *bright) {
        return QColor(lightSurface ? bright : dark);
    };
    const auto neutral = tone("#ffffff", "#000000");
    const auto layer = [&](const char *dark, const char *bright, double opacity) {
        return customBackground.isValid() ? blend(canvas, neutral, opacity) : tone(dark, bright);
    };
    const auto rail = layer("#1b201e", "#ecf2ef", 0.04);
    const auto surface = customBackground.isValid() && lightSurface ? blend(canvas, Qt::white, 0.24)
                                                                   : layer("#212724", "#ffffff", 0.07);
    const auto base = customBackground.isValid() && lightSurface ? blend(canvas, Qt::white, 0.17)
                                                                : layer("#1d2220", "#ffffff", 0.045);
    const auto alternateRow = layer("#222925", "#eff4f1", 0.075);
    const auto border = layer("#39463f", "#c9d6cf", 0.19);
    const auto control = layer("#778b80", "#687d72", 0.46);
    const auto defaultText = tone("#edf5f0", "#192a23");
    const auto text = customForeground.isValid() ? customForeground
        : appearanceColorContrast(canvas, defaultText) >= 4.5 ? defaultText : neutral;
    const auto muted = (customBackground.isValid() || customForeground.isValid())
        ? blend(canvas, text, 0.78) : tone("#a9bcb1", "#50665c");
    const auto hover = layer("#2d3832", "#e4eee8", 0.11);
    const auto accentKey = appearance.value("accent").toString("cirnet");
    const auto accent = appearanceAccent(accentKey, lightSurface, appearance.value("custom_accent").toString());
    const auto onAccent = appearanceColorContrast(accent, Qt::black) >= appearanceColorContrast(accent, Qt::white)
        ? QColor(Qt::black) : QColor(Qt::white);
    const auto selected = blend(canvas, accent, lightSurface ? 0.17 : 0.21);
    const auto scrollThumb = accentKey == "cirnet" && !customBackground.isValid()
        ? tone("#618675", "#638878") : blend(control, accent, 0.40);
    const auto contrastAccent = appearanceColorContrast(canvas, accent) >= 3.0 ? accent : neutral;
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
    palette.setColor(QPalette::Link, contrastAccent);
    for (const auto role : {QPalette::Text, QPalette::WindowText, QPalette::ButtonText})
        palette.setColor(QPalette::Disabled, role, QColor(muted));
    palette.setColor(QPalette::Disabled, QPalette::Highlight, QColor(control));
    palette.setColor(QPalette::Disabled, QPalette::HighlightedText, QColor(onAccent));
    if (QApplication::palette() != palette)
        QApplication::setPalette(palette);
    const auto fixedFont = QFontDatabase::systemFont(QFontDatabase::FixedFont);
    QFont font = appearance.value("ui_font") == QStringLiteral("monospace") ? fixedFont : systemFont;
    const auto family = appearance.value("font_family").toString();
    const bool customFont = !family.isEmpty() && QFontDatabase::hasFamily(family)
        && QFontDatabase::isSmoothlyScalable(family);
    if (customFont)
        font.setFamily(family);
    const int interfacePx = qBound(10, appearance.value("interface_px").toInt(qRound(13 * textScale / 100.0)), 26);
    font.setPixelSize(interfacePx);
    if (QApplication::font() != font)
        QApplication::setFont(font);

    static const auto templateStyle = [] {
        QFile file(CIRNETRULES_RESOURCE_ROOT "/theme.qss");
        return file.open(QIODevice::ReadOnly) ? QString::fromUtf8(file.readAll()) : QString{};
    }();
    if (!templateStyle.isEmpty()) {
        auto style = templateStyle;
        const auto scaled = [interfacePx](int size) { return QString::number(qRound(size * interfacePx / 13.0)); };
        const auto sizeFor = [interfacePx, &appearance](const char *pixelKey, const char *scaleKey) {
            const auto scale = appearance.value(QLatin1StringView(scaleKey)).toInt();
            const auto fallback = scale == 0 ? interfacePx : qRound(13 * scale / 100.0);
            return QString::number(qBound(10, appearance.value(QLatin1StringView(pixelKey)).toInt(fallback), 26));
        };
        const auto cssFamily = [](QString name) {
            name.replace(QStringLiteral("\\"), QStringLiteral("\\\\"));
            name.replace(QStringLiteral("\""), QStringLiteral("\\\""));
            return name;
        };
        const QList<QPair<QString, QString>> tokens{
            {"CANVAS", canvas.name()}, {"RAIL", rail.name()}, {"SURFACE", surface.name()}, {"BASE", base.name()},
            {"ALTERNATE_ROW", alternateRow.name()}, {"BORDER", border.name()}, {"CONTROL", control.name()},
            {"TEXT", text.name()}, {"MUTED", muted.name()}, {"HOVER", hover.name()}, {"SELECTED", selected.name()},
            {"ACCENT", contrastAccent.name()}, {"PRIMARY", accent.name()}, {"ON_ACCENT", onAccent.name()},
            {"SCROLL_THUMB", scrollThumb.name()},
            {"PRIMARY_HOVER", blend(accent, onAccent == Qt::black ? Qt::white : Qt::black, 0.12).name()},
            {"PRIMARY_PRESSED", blend(accent, onAccent, 0.14).name()},
            {"DANGER", danger.name()}, {"WARNING", warning.name()}, {"TITLE_SIZE", scaled(24)},
            {"SECTION_SIZE", scaled(15)}, {"META_SIZE", scaled(12)},
            {"CONTENT_SIZE", sizeFor("conversation_px", "content_scale")}, {"CODE_SIZE", sizeFor("code_px", "code_scale")},
            {"UI_FAMILY", cssFamily(font.family())},
            {"CODE_FAMILY", cssFamily(customFont ? font.family() : fixedFont.family())},
            {"VERTICAL_PAD", QString::number(padding)}
        };
        for (const auto &[token, value] : tokens)
            style.replace("@" + token + "@", value);
        if (qApp->styleSheet() != style)
            qApp->setStyleSheet(style);
    }
    emit changed();
}
