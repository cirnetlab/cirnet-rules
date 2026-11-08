#pragma once

#include <QDeadlineTimer>
#include <QHash>
#include <QJsonObject>
#include <QProcess>
#include <QStringList>
#include <QTimer>
#include <atomic>
#include <functional>
#include <memory>

class QThread;

class RuntimeClient final : public QObject {
    Q_OBJECT
public:
    using Reply = std::function<void(const QJsonObject &, const QString &)>;

    RuntimeClient(QString executable, QStringList arguments, QObject *parent = nullptr);
    ~RuntimeClient() override;
    static QString siblingExecutable();
    static QString validateExecutable(const QString &path, bool requirePin = false);
    bool isRunning() const;
    void start();
    void stop();
    void request(const QString &method, QJsonObject params, Reply reply);

signals:
    void ready();
    void disconnected(const QString &reason);
    void stopped();

private:
    struct Pending {
        Reply callback;
        QDeadlineTimer deadline;
    };
    void receive();
    void expireRequests();
    void rejectConnection(const QString &reason);
    void failPending(const QString &reason);
    void processFinished();
    void finishProcess();

    QString executable_;
    QStringList arguments_;
    QProcess process_;
    QByteArray buffered_;
    QHash<QString, Pending> pending_;
    QTimer expiry_;
    QThread *validation_ = nullptr;
    std::shared_ptr<std::atomic_bool> cancelValidation_;
    bool drainScheduled_ = false;
    bool draining_ = false;
    bool processExited_ = false;
    quint64 generation_ = 0;
    bool closing_ = false;
    bool disconnected_ = false;
};
