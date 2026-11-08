#pragma once

#include <QDateTime>
#include <QList>
#include <QObject>
#include <QPointer>

class QFrame;
class QListWidget;
class QWidget;

class NotificationCenter final : public QObject {
    Q_OBJECT
public:
    explicit NotificationCenter(QWidget &host);
    void post(const QString &text, const QString &severity);
    void setLevel(const QString &level);
    void attachHistory(QListWidget &history);
    void markHistoryRead();
    void clearRead();
    void refreshIcons();

protected:
    bool eventFilter(QObject *object, QEvent *event) override;

private:
    struct Notice {
        quint64 id;
        QString text;
        QString severity;
        QDateTime time;
        bool read = false;
    };
    struct Toast {
        QFrame *widget;
        quint64 id;
        int remaining;
    };
    void fill();
    void reposition();
    void dismiss(quint64 id);
    void updateHistory();

    QWidget &host_;
    QPointer<QListWidget> history_;
    QList<Notice> records_;
    QList<Notice> queued_;
    QList<Toast> visible_;
    QString level_ = QStringLiteral("all");
    quint64 sequence_ = 0;
};
