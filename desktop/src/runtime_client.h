#pragma once

#include <QDeadlineTimer>
#include <QElapsedTimer>
#include <QHash>
#include <QJsonObject>
#include <QProcess>
#include <QStringList>
#include <QTimer>
#include <array>
#include <atomic>
#include <functional>
#include <memory>
#include <optional>

class QThread;

class RuntimeClient final : public QObject {
    Q_OBJECT
public:
    using Reply = std::function<void(const QJsonObject &, const QString &)>;
    enum class ViewRead { ConversationDetail, ConversationEvents };

    RuntimeClient(QString executable, QStringList arguments, QObject *parent = nullptr);
    ~RuntimeClient() override;
    static QString siblingExecutable();
    static QString validateExecutable(const QString &path, bool requirePin = false);
    bool isRunning() const;
    QJsonObject runtimeInfo() const { return runtimeInfo_; }
    void start();
    void stop();
    void request(const QString &method, QJsonObject params, Reply reply);
    void latestRead(ViewRead purpose, QJsonObject params, Reply reply);
    void cancelLatestRead(ViewRead purpose);

signals:
    void ready();
    void disconnected(const QString &reason);
    void stopped();

private:
    struct Pending {
        Reply callback;
        QDeadlineTimer deadline;
        QElapsedTimer submitted;
        std::optional<ViewRead> purpose;
        bool obsolete = false;
        bool cancelSent = false;
    };
    struct ReadIntent {
        QJsonObject params;
        Reply callback;
        QDeadlineTimer deadline;
        QElapsedTimer enqueued;
    };
    void submit(const QString &method, QJsonObject params, Pending pending, qint64 queueUs = 0);
    void replaceRead(ViewRead purpose, std::optional<ReadIntent> next);
    void scheduleReads();
    void dispatchReads();
    bool hasQueuedRead() const;
    void receive();
    void expireRequests();
    void rejectConnection(const QString &reason);
    void failPending(const QString &reason);
    void processFinished();
    void finishProcess();
    void traceLine(QByteArray line);
    void readRuntimeTrace();

    QString executable_;
    QStringList arguments_;
    QProcess *const process_;
    QJsonObject runtimeInfo_;
    QByteArray buffered_;
    QByteArray traceBuffer_;
    qsizetype traceBudget_ = 0;
    QHash<QString, Pending> pending_;
    std::array<std::optional<ReadIntent>, 2> queuedReads_;
    QTimer expiry_;
    QTimer readDispatch_;
    QThread *validation_ = nullptr;
    std::shared_ptr<std::atomic_bool> cancelValidation_;
    bool drainScheduled_ = false;
    bool draining_ = false;
    bool processExited_ = false;
    quint64 generation_ = 0;
    bool closing_ = false;
    bool disconnected_ = false;
};
