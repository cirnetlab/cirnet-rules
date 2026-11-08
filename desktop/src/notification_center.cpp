#include "notification_center.h"
#include "theme.h"

#include <QApplication>
#include <QEvent>
#include <QFrame>
#include <QFontMetrics>
#include <QGraphicsOpacityEffect>
#include <QHBoxLayout>
#include <QLabel>
#include <QListWidget>
#include <QPainter>
#include <QPalette>
#include <QPropertyAnimation>
#include <QStringList>
#include <QTextLayout>
#include <QTimer>
#include <QToolButton>
#include <algorithm>
#include <iterator>

namespace {
constexpr int inset = 18;
constexpr int spacing = 8;
constexpr int expiryInterval = 100;

bool important(const QString &severity)
{
    return severity == "warning" || severity == "error";
}

UiIcon severityIcon(const QString &severity)
{
    if (severity == "error")
        return UiIcon::Error;
    if (severity == "warning")
        return UiIcon::Warning;
    if (severity == "success")
        return UiIcon::Success;
    return UiIcon::Info;
}

class ToastIcon final : public QLabel {
public:
    explicit ToastIcon(UiIcon symbol) : icon_(uiIcon(symbol))
    {
        setFixedSize(22, 22);
        setAttribute(Qt::WA_TransparentForMouseEvents);
    }

protected:
    void paintEvent(QPaintEvent *event) override
    {
        QLabel::paintEvent(event);
        QPainter painter(this);
        icon_.paint(&painter, rect().adjusted(2, 2, -2, -2));
    }

private:
    const QIcon icon_;
};

QString plainMessage(const QString &source)
{
    QString result;
    for (const QChar character : source.first(qMin<qsizetype>(source.size(), 480))) {
        if (character.isPrint())
            result.append(character);
        else if (character.isSpace())
            result.append(' ');
    }
    return result.simplified();
}

QString summarize(const QString &text, const QFont &font, int width)
{
    QTextLayout layout(text, font);
    QTextOption options;
    options.setWrapMode(QTextOption::WrapAtWordBoundaryOrAnywhere);
    layout.setTextOption(options);
    layout.beginLayout();
    QStringList lines;
    for (int row = 0; row < 3; ++row) {
        auto line = layout.createLine();
        if (!line.isValid())
            break;
        line.setLineWidth(width);
        const auto from = line.textStart();
        if (row == 2 && from + line.textLength() < text.size())
            lines.append(QFontMetrics(font).elidedText(text.mid(from), Qt::ElideRight, width));
        else
            lines.append(text.mid(from, line.textLength()).trimmed());
    }
    layout.endLayout();
    return lines.join(u'\n');
}

void fadeIn(QFrame *card)
{
    auto *effect = new QGraphicsOpacityEffect(card);
    effect->setOpacity(0.65);
    card->setGraphicsEffect(effect);
    auto *animation = new QPropertyAnimation(effect, "opacity", effect);
    animation->setDuration(140);
    animation->setEndValue(1.0);
    QObject::connect(animation, &QPropertyAnimation::finished, card, [card, current = QPointer<QGraphicsOpacityEffect>(effect)] {
        QTimer::singleShot(0, card, [card, current] {
            if (current && card->graphicsEffect() == current)
                card->setGraphicsEffect(nullptr);
        });
    });
    animation->start();
}
}

NotificationCenter::NotificationCenter(QWidget &host) : QObject(&host), host_(host)
{
    host_.installEventFilter(this);
    expiry_ = new QTimer(this);
    expiry_->setInterval(expiryInterval);
    connect(expiry_, &QTimer::timeout, this, [this] {
        QList<quint64> expired;
        for (auto &toast : visible_) {
            const auto *focus = QApplication::focusWidget();
            if (toast.remaining > 0 && toast.widget->isVisible() && !toast.widget->underMouse() &&
                !(focus && toast.widget->isAncestorOf(focus))) {
                toast.remaining -= expiryInterval;
                if (toast.remaining <= 0)
                    expired.append(toast.id);
            }
        }
        for (auto id : expired)
            dismiss(id);
    });
}

void NotificationCenter::setReducedMotion(bool reduced)
{
    reducedMotion_ = reduced;
    if (reducedMotion_) {
        for (const auto &toast : visible_)
            toast.widget->setGraphicsEffect(nullptr);
    }
}

void NotificationCenter::setAvoidanceWidget(QWidget *widget)
{
    if (avoidance_)
        avoidance_->removeEventFilter(this);
    avoidance_ = widget;
    if (avoidance_)
        avoidance_->installEventFilter(this);
    reposition();
}

void NotificationCenter::post(const QString &text, const QString &severity)
{
    const auto plain = plainMessage(text);
    if (plain.isEmpty())
        return;
    const auto now = QDateTime::currentDateTime();
    for (const auto &old : records_) {
        if (old.time.msecsTo(now) > 5000)
            break;
        if (old.text == plain && old.severity == severity)
            return;
    }
    const Notice notice{++sequence_, plain, severity, now};
    records_.prepend(notice);
    if (records_.size() > 100)
        records_.removeLast();
    if (level_ == "all" || important(severity)) {
        if (queued_.size() >= 12) {
            const auto regular = std::find_if(queued_.cbegin(), queued_.cend(),
                                              [](const Notice &item) { return !important(item.severity); });
            if (regular != queued_.cend())
                queued_.removeAt(std::distance(queued_.cbegin(), regular));
            else if (important(severity))
                queued_.removeFirst();
        }
        if (queued_.size() < 12)
            queued_.append(notice);
        fill();
    }
    updateHistory();
}

void NotificationCenter::setLevel(const QString &level)
{
    level_ = level;
    if (level_ == "important")
        queued_.removeIf([](const Notice &item) { return !important(item.severity); });
}

void NotificationCenter::attachHistory(QListWidget &history)
{
    history_ = &history;
    updateHistory();
}

void NotificationCenter::markHistoryRead()
{
    for (auto &record : records_)
        record.read = true;
    updateHistory();
}

void NotificationCenter::clearRead()
{
    records_.removeIf([](const Notice &record) { return record.read; });
    updateHistory();
}

void NotificationCenter::fill()
{
    QList<QFrame *> created;
    while (visible_.size() < 3 && !queued_.isEmpty()) {
        const auto notice = queued_.takeFirst();
        auto *card = new QFrame(&host_);
        card->setObjectName("toast");
        card->setProperty("severity", notice.severity);
        card->setAccessibleName(severityLabel(notice.severity));
        auto *layout = new QHBoxLayout(card);
        layout->setContentsMargins(12, 8, 8, 8);
        layout->setSpacing(10);
        auto *icon = new ToastIcon(severityIcon(notice.severity));
        icon->setObjectName("toastIcon");
        layout->addWidget(icon, 0, Qt::AlignVCenter);
        auto *body = new QLabel(notice.text);
        body->setObjectName("toastBody");
        body->setTextFormat(Qt::PlainText);
        body->setWordWrap(false);
        body->setToolTip(notice.text.toHtmlEscaped());
        body->setMinimumWidth(0);
        body->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        layout->addWidget(body, 1);
        auto *close = new QToolButton;
        close->setObjectName("dismissToast");
        close->setFixedSize(30, 30);
        close->setIconSize(QSize(16, 16));
        close->setToolTip(tr("Dismiss notification"));
        close->setAccessibleName(tr("Dismiss notification"));
        layout->addWidget(close, 0, Qt::AlignVCenter);
        connect(close, &QToolButton::clicked, this, [this, id = notice.id] { dismiss(id); });
        visible_.append({card, notice.id, important(notice.severity) ? -1 : 5400, notice.text});
        created.append(card);
    }
    refreshIcons();
    if (!reducedMotion_) {
        for (auto *card : created) {
            if (card->isVisible())
                fadeIn(card);
        }
    }
}

void NotificationCenter::reposition()
{
    const int width = qMax(0, qMin(host_.width() - 2 * inset, 360));
    const int x = host_.width() - inset - width;
    int bottom = host_.height() - inset;
    if (avoidance_ && host_.isAncestorOf(avoidance_) && avoidance_->isVisibleTo(&host_)) {
        const QRect obstacle(avoidance_->mapTo(&host_, QPoint{}), avoidance_->size());
        if (obstacle.right() >= x && obstacle.left() < x + width)
            bottom = qMin(bottom, obstacle.top() - spacing);
    }
    bool hasExpiry = false;
    for (auto it = visible_.crbegin(); it != visible_.crend(); ++it) {
        const auto &toast = *it;
        toast.widget->ensurePolished();
        toast.widget->setFixedWidth(width);
        auto *body = toast.widget->findChild<QLabel *>("toastBody");
        body->ensurePolished();
        const auto icon = toast.widget->findChild<QLabel *>("toastIcon")->size();
        const auto button = toast.widget->findChild<QToolButton *>("dismissToast")->size();
        const auto *layout = toast.widget->layout();
        const auto margins = layout->contentsMargins();
        const auto frame = toast.widget->contentsMargins();
        const int textWidth = width - margins.left() - margins.right() - icon.width() -
                              button.width() - 2 * layout->spacing() - frame.left() - frame.right() - 2;
        if (textWidth < 24) {
            toast.widget->hide();
            continue;
        }
        const auto summary = summarize(toast.text, body->font(), textWidth);
        if (body->text() != summary)
            body->setText(summary);
        const int textHeight = body->fontMetrics().height() + int(summary.count(u'\n')) * body->fontMetrics().lineSpacing();
        const int rowHeight = qMax(qMax(icon.height(), button.height()), textHeight);
        const int height = margins.top() + rowHeight + margins.bottom() + frame.top() + frame.bottom();
        if (bottom - height < inset) {
            toast.widget->hide();
            continue;
        }
        toast.widget->setGeometry(x, bottom - height, width, height);
        toast.widget->show();
        toast.widget->raise();
        bottom -= height + spacing;
        hasExpiry |= toast.remaining > 0 && toast.widget->isVisible();
    }
    if (hasExpiry && !expiry_->isActive())
        expiry_->start();
    else if (!hasExpiry)
        expiry_->stop();
}

void NotificationCenter::dismiss(quint64 id)
{
    for (qsizetype index = 0; index < visible_.size(); ++index) {
        if (visible_[index].id == id) {
            auto *card = visible_.takeAt(index).widget;
            card->hide();
            card->deleteLater();
            break;
        }
    }
    fill();
}

QString NotificationCenter::severityLabel(const QString &severity)
{
    if (severity == "error")
        return tr("Error");
    if (severity == "warning")
        return tr("Warning");
    if (severity == "success")
        return tr("Success");
    return tr("Information");
}

void NotificationCenter::updateHistory()
{
    if (!history_)
        return;
    history_->clear();
    for (auto &item : records_) {
        if (history_->isVisible())
            item.read = true;
        auto *row = new QListWidgetItem(tr("%1 · %2 — %3")
            .arg(item.time.time().toString("HH:mm"), severityLabel(item.severity), item.text), history_);
        row->setToolTip(item.text.toHtmlEscaped());
    }
}

void NotificationCenter::refreshIcons()
{
    const auto muted = QApplication::palette().color(QPalette::PlaceholderText);
    for (const auto &toast : visible_) {
        toast.widget->findChild<QLabel *>("toastIcon")->update();
        toast.widget->findChild<QToolButton *>("dismissToast")->setIcon(uiIcon(UiIcon::Close, muted));
    }
    reposition();
}

bool NotificationCenter::eventFilter(QObject *object, QEvent *event)
{
    const auto type = event->type();
    if ((object == &host_ || object == avoidance_.data()) && !repositionPending_ &&
        (type == QEvent::Resize || type == QEvent::Move || type == QEvent::Show ||
         type == QEvent::Hide || type == QEvent::LayoutRequest)) {
        repositionPending_ = true;
        QTimer::singleShot(0, this, [this] {
            repositionPending_ = false;
            reposition();
        });
    }
    return QObject::eventFilter(object, event);
}
