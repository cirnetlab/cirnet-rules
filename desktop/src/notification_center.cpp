#include "notification_center.h"
#include "theme.h"

#include <QApplication>
#include <QEvent>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QListWidget>
#include <QTimer>
#include <QToolButton>
#include <algorithm>
#include <iterator>

namespace {
bool important(const QString &severity)
{
    return severity == "warning" || severity == "error";
}

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
}

NotificationCenter::NotificationCenter(QWidget &host) : QObject(&host), host_(host)
{
    host_.installEventFilter(this);
    auto *timer = new QTimer(this);
    timer->setInterval(100);
    connect(timer, &QTimer::timeout, this, [this] {
        QList<quint64> expired;
        for (auto &toast : visible_) {
            const auto *focus = QApplication::focusWidget();
            if (toast.remaining > 0 && !toast.widget->underMouse() &&
                !(focus && toast.widget->isAncestorOf(focus))) {
                toast.remaining -= 100;
                if (toast.remaining <= 0)
                    expired.append(toast.id);
            }
        }
        for (auto id : expired)
            dismiss(id);
    });
    timer->start();
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
    while (visible_.size() < 3 && !queued_.isEmpty()) {
        const auto notice = queued_.takeFirst();
        auto *card = new QFrame(&host_);
        card->setObjectName("toast");
        card->setProperty("severity", notice.severity);
        auto *layout = new QHBoxLayout(card);
        layout->setContentsMargins(10, 8, 8, 8);
        auto *icon = new QLabel;
        icon->setObjectName("toastIcon");
        layout->addWidget(icon, 0, Qt::AlignTop);
        auto *body = new QLabel(notice.text);
        body->setObjectName("toastBody");
        body->setTextFormat(Qt::PlainText);
        body->setWordWrap(true);
        body->setMinimumWidth(0);
        body->setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Preferred);
        layout->addWidget(body, 1);
        auto *close = new QToolButton;
        close->setObjectName("dismissToast");
        close->setToolTip(tr("Dismiss notification"));
        close->setAccessibleName(tr("Dismiss notification"));
        layout->addWidget(close, 0, Qt::AlignTop);
        connect(close, &QToolButton::clicked, this, [this, id = notice.id] { dismiss(id); });
        visible_.append({card, notice.id, notice.severity == "error" ? -1 : 5400});
        card->show();
    }
    refreshIcons();
}

void NotificationCenter::reposition()
{
    const int width = qBound(160, host_.width() - 36, 360);
    int y = 18;
    for (const auto &toast : visible_) {
        toast.widget->setFixedWidth(width);
        const auto *body = toast.widget->findChild<QLabel *>("toastBody");
        const auto icon = toast.widget->findChild<QLabel *>("toastIcon")->sizeHint();
        const auto button = toast.widget->findChild<QToolButton *>("dismissToast")->sizeHint();
        const auto *layout = toast.widget->layout();
        const auto margins = layout->contentsMargins();
        const int textWidth = width - margins.left() - margins.right() - icon.width() -
                              button.width() - 2 * layout->spacing();
        const int rowHeight = qMax(qMax(icon.height(), button.height()), body->heightForWidth(qMax(1, textWidth)));
        toast.widget->setGeometry(qMax(0, host_.width() - width - 18), y, width,
                                  margins.top() + rowHeight + margins.bottom());
        toast.widget->raise();
        y += toast.widget->height() + 8;
    }
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

void NotificationCenter::updateHistory()
{
    if (!history_)
        return;
    history_->clear();
    for (auto &item : records_) {
        if (history_->isVisible())
            item.read = true;
        const auto severity = item.severity == "error" ? tr("Error") :
                              item.severity == "warning" ? tr("Warning") :
                              item.severity == "success" ? tr("Success") : tr("Information");
        auto *row = new QListWidgetItem(tr("%1 · %2 — %3")
                                            .arg(item.time.time().toString("HH:mm"), severity, item.text), history_);
        row->setToolTip(item.text.toHtmlEscaped());
    }
}

void NotificationCenter::refreshIcons()
{
    for (const auto &toast : visible_) {
        const auto severity = toast.widget->property("severity").toString();
        auto *icon = toast.widget->findChild<QLabel *>("toastIcon");
        icon->setPixmap(uiIcon(important(severity) ? UiIcon::Warning :
                              severity == "success" ? UiIcon::Success : UiIcon::Info).pixmap(18));
        toast.widget->findChild<QToolButton *>("dismissToast")->setIcon(uiIcon(UiIcon::Close));
    }
    reposition();
}

bool NotificationCenter::eventFilter(QObject *object, QEvent *event)
{
    if (object == &host_ && (event->type() == QEvent::Resize || event->type() == QEvent::LayoutRequest))
        reposition();
    return QObject::eventFilter(object, event);
}
