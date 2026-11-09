#include "build_info.h"
#include "runtime_client.h"

#include <QCoreApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QScopedValueRollback>
#include <QThread>
#include <QUuid>
#include <utility>

namespace {
constexpr qsizetype frameLimit = 256 * 1024;
constexpr qsizetype pendingLimit = 32;
constexpr qsizetype writeLimit = 2 * frameLimit;
constexpr int framesPerSlice = 8;
constexpr qint64 drainBudgetMs = 5;
constexpr qint64 requestTimeoutMs = 15000;

QString checkExecutable(const QString &path, bool requirePin, const std::atomic_bool *cancelled = nullptr)
{
    const QFileInfo info(path);
    if (!info.isAbsolute() || !info.isFile() || !info.isExecutable() || info.isSymLink() ||
        info.canonicalFilePath().isEmpty())
        return RuntimeClient::tr("The local runtime executable is missing or invalid.");
    const QString pinPath = path + QStringLiteral(".sha256");
    const QFileInfo pinInfo(pinPath);
    if (!pinInfo.exists() && !pinInfo.isSymLink())
        return requirePin ? RuntimeClient::tr("The local runtime checksum is missing.") : QString{};
    QFile pin(pinPath);
    if (pinInfo.isSymLink() || !pinInfo.isFile() || pinInfo.size() > 128 || !pin.open(QIODevice::ReadOnly))
        return RuntimeClient::tr("The local runtime checksum is invalid.");
    QFile executable(info.canonicalFilePath());
    if (!executable.open(QIODevice::ReadOnly) || executable.size() > 128 * 1024 * 1024)
        return RuntimeClient::tr("The local runtime could not be verified.");
    QCryptographicHash hash(QCryptographicHash::Sha256);
    while (!executable.atEnd()) {
        if (cancelled && cancelled->load(std::memory_order_relaxed))
            return {};
        const auto chunk = executable.read(256 * 1024);
        if (chunk.isEmpty())
            return RuntimeClient::tr("The local runtime could not be verified.");
        hash.addData(chunk);
    }
    if (hash.result().toHex() != pin.readAll().trimmed().toLower())
        return RuntimeClient::tr("The local runtime checksum does not match.");
    return {};
}
}

RuntimeClient::RuntimeClient(QString executable, QStringList arguments, QObject *parent)
    : QObject(parent), executable_(std::move(executable)), arguments_(std::move(arguments))
{
    process_.setProcessChannelMode(QProcess::SeparateChannels);
    process_.setStandardErrorFile(QProcess::nullDevice());
    expiry_.setInterval(250);
    connect(&expiry_, &QTimer::timeout, this, &RuntimeClient::expireRequests);
    connect(&process_, &QProcess::started, this, [this] {
        request("runtime.info", {}, [this](const auto &info, const auto &error) {
            if (closing_ || disconnected_)
                return;
            if (!error.isEmpty() || !compatibleRuntime(info)) {
                rejectConnection(error.isEmpty() ? tr("Dashboard and runtime identities differ.") : error);
                return;
            }
            runtimeInfo_ = info;
            emit ready();
        });
    });
    connect(&process_, &QProcess::readyReadStandardOutput, this, &RuntimeClient::receive);
    connect(&process_, &QProcess::finished, this, &RuntimeClient::processFinished);
    connect(&process_, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        if (error == QProcess::FailedToStart)
            rejectConnection(tr("The runtime could not start."));
        else if (error == QProcess::ReadError || error == QProcess::WriteError)
            rejectConnection(tr("The runtime transport failed."));
    });
}

RuntimeClient::~RuntimeClient()
{
    process_.disconnect(this);
    if (cancelValidation_)
        cancelValidation_->store(true, std::memory_order_relaxed);
    if (process_.state() != QProcess::NotRunning)
        process_.kill();
}

QString RuntimeClient::siblingExecutable()
{
#ifdef Q_OS_WIN
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral(CIRNETRULES_RUNTIME_BASENAME ".exe"));
#else
    return QDir(QCoreApplication::applicationDirPath()).filePath(QStringLiteral(CIRNETRULES_RUNTIME_BASENAME));
#endif
}

QString RuntimeClient::validateExecutable(const QString &path, bool requirePin)
{
    return checkExecutable(path, requirePin);
}

bool RuntimeClient::isRunning() const
{
    return validation_ || processExited_ || process_.state() != QProcess::NotRunning;
}

void RuntimeClient::start()
{
    if (isRunning())
        return;
    ++generation_;
    closing_ = false;
    disconnected_ = false;
    buffered_.clear();
    runtimeInfo_ = {};
    drainScheduled_ = false;
    const bool isAppRuntime = executable_ == siblingExecutable();
    struct Validation { QString error, program, directory; };
    const auto outcome = std::make_shared<Validation>();
    cancelValidation_ = std::make_shared<std::atomic_bool>(false);
    validation_ = QThread::create([path = executable_, appPath = QCoreApplication::applicationFilePath(),
                                   isAppRuntime, outcome, stop = cancelValidation_] {
        outcome->error = checkExecutable(path, isAppRuntime, stop.get());
        if (!outcome->error.isEmpty() || stop->load(std::memory_order_relaxed))
            return;
        const QFileInfo info(path);
        outcome->program = info.canonicalFilePath();
        outcome->directory = info.canonicalPath();
        if (isAppRuntime && outcome->directory != QFileInfo(appPath).canonicalPath())
            outcome->error = RuntimeClient::tr("The local runtime is outside the application directory.");
    });
    connect(validation_, &QThread::finished, this, [this, outcome] {
        validation_ = nullptr;
        cancelValidation_.reset();
        if (closing_) {
            emit stopped();
            return;
        }
        if (!outcome->error.isEmpty()) {
            rejectConnection(outcome->error);
            return;
        }
        process_.setWorkingDirectory(outcome->directory);
        process_.start(outcome->program, arguments_);
    });
    connect(validation_, &QThread::finished, validation_, &QObject::deleteLater);
    validation_->start();
}

void RuntimeClient::stop()
{
    if (!isRunning()) {
        emit stopped();
        return;
    }
    if (closing_)
        return;
    closing_ = true;
    if (cancelValidation_) {
        cancelValidation_->store(true, std::memory_order_relaxed);
        return;
    }
    if (processExited_)
        return;
    process_.closeWriteChannel();
    const auto generation = generation_;
    QTimer::singleShot(1800, this, [this, generation] {
        if (generation == generation_ && isRunning())
            process_.terminate();
    });
    QTimer::singleShot(2600, this, [this, generation] {
        if (generation == generation_ && isRunning())
            process_.kill();
    });
}

void RuntimeClient::request(const QString &method, QJsonObject params, Reply reply)
{
    const QString id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    const auto request = QJsonDocument(QJsonObject{
        {"v", CIRNETRULES_IPC_PROTOCOL}, {"id", id}, {"method", QStringLiteral(CIRNETRULES_API_NAMESPACE) + method}, {"params", std::move(params)}
    }).toJson(QJsonDocument::Compact) + '\n';
    QString error;
    if (process_.state() != QProcess::Running || closing_ || disconnected_)
        error = tr("The runtime is disconnected.");
    else if (runtimeInfo_.isEmpty() && method != "runtime.info")
        error = tr("Wait for runtime identity verification before sending operations.");
    else if (pending_.size() >= pendingLimit || process_.bytesToWrite() > writeLimit)
        error = tr("The runtime request queue is full.");
    else if (request.size() > frameLimit)
        error = tr("The runtime request exceeds the size limit.");
    if (!error.isEmpty()) {
        if (reply)
            reply({}, error);
        return;
    }
    pending_.insert(id, Pending{std::move(reply), QDeadlineTimer(requestTimeoutMs)});
    if (!expiry_.isActive())
        expiry_.start();
    if (process_.write(request) != request.size()) {
        rejectConnection(tr("The runtime transport failed."));
        return;
    }
}

void RuntimeClient::receive()
{
    if (drainScheduled_ || draining_)
        return;
    const QScopedValueRollback guard(draining_, true);
    QElapsedTimer elapsed;
    elapsed.start();
    int frames = 0;
    while (!disconnected_ && frames < framesPerSlice && elapsed.elapsed() < drainBudgetMs) {
        auto newline = buffered_.indexOf('\n');
        if (newline < 0 && process_.bytesAvailable() > 0) {
            buffered_.append(process_.read(32768));
            newline = buffered_.indexOf('\n');
        }
        if (newline < 0) {
            if (buffered_.size() > frameLimit) {
                rejectConnection(tr("The runtime response exceeds the size limit."));
                return;
            }
            if (process_.bytesAvailable() == 0)
                break;
            continue;
        }
        ++frames;
        if (newline > frameLimit) {
            rejectConnection(tr("The runtime response exceeds the size limit."));
            return;
        }
        const auto bytes = buffered_.left(newline);
        buffered_.remove(0, newline + 1);
        QJsonParseError parse;
        const auto document = QJsonDocument::fromJson(bytes, &parse);
        const auto response = document.object();
        const auto id = response.value("id").toString();
        const auto ok = response.value("ok");
        const auto result = response.value("result");
        const auto error = response.value("error");
        if (parse.error == QJsonParseError::NoError && document.isObject() &&
            response.value("v").isDouble() && response.value("v").toInt() != CIRNETRULES_IPC_PROTOCOL) {
            rejectConnection(tr("The runtime protocol is incompatible. Use the runtime installed with %1.").arg(QStringLiteral(CIRNETRULES_PRODUCT_NAME)));
            return;
        }
        if (parse.error != QJsonParseError::NoError || !document.isObject() ||
            response.value("v").toInt() != CIRNETRULES_IPC_PROTOCOL || id.isEmpty() || !ok.isBool() ||
            (ok.toBool() ? !result.isObject() : !error.isObject())) {
            rejectConnection(tr("The runtime sent an invalid response."));
            return;
        }
        auto entry = pending_.find(id);
        if (entry == pending_.end())
            continue;
        auto callback = std::move(entry.value().callback);
        pending_.erase(entry);
        if (pending_.isEmpty())
            expiry_.stop();
        if (callback) {
            QString message;
            auto payload = result.toObject();
            if (!ok.toBool()) {
                const auto details = error.toObject();
                payload = details.value("details").toObject();
                payload.insert("code", details.value("code"));
                message = details.value("message").toString().simplified().left(450);
                if (message.isEmpty())
                    message = tr("The runtime rejected the operation.");
            }
            callback(payload, message);
        }
    }
    if (!disconnected_ && (process_.bytesAvailable() > 0 || buffered_.contains('\n'))) {
        drainScheduled_ = true;
        const auto generation = generation_;
        QTimer::singleShot(0, this, [this, generation] {
            if (generation == generation_) {
                drainScheduled_ = false;
                receive();
            }
        });
    } else if (processExited_) {
        finishProcess();
    }
}

void RuntimeClient::expireRequests()
{
    QList<Reply> expired;
    for (auto it = pending_.begin(); it != pending_.end();) {
        if (it.value().deadline.hasExpired()) {
            expired.append(std::move(it.value().callback));
            it = pending_.erase(it);
        } else {
            ++it;
        }
    }
    if (pending_.isEmpty())
        expiry_.stop();
    for (auto &callback : expired)
        if (callback)
            callback({}, tr("The runtime request timed out. Check the saved state before retrying."));
}

void RuntimeClient::rejectConnection(const QString &reason)
{
    if (disconnected_)
        return;
    disconnected_ = true;
    buffered_.clear();
    failPending(reason);
    if (process_.state() != QProcess::NotRunning)
        process_.kill();
    if (!closing_)
        emit disconnected(reason);
    if (processExited_)
        finishProcess();
}

void RuntimeClient::failPending(const QString &reason)
{
    auto queued = std::exchange(pending_, {});
    expiry_.stop();
    for (auto &reply : queued) {
        if (reply.callback)
            reply.callback({}, reason);
    }
}

void RuntimeClient::processFinished()
{
    processExited_ = true;
    receive();
}

void RuntimeClient::finishProcess()
{
    processExited_ = false;
    if (closing_) {
        failPending(tr("The runtime stopped."));
        emit stopped();
    } else {
        rejectConnection(tr("The runtime stopped. Reconnect to restore saved state."));
    }
}
